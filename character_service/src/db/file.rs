use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{self, create_dir_all, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
};

use protobuf::{Message, SingularPtrField};
use tracing::{error, trace, warn};

use crate::{
    db::errors::{Error, ErrorCode},
    proto::storage::CharacterMetadata,
};

use proto_rs::{character::Character, characters::RevisionRead};

// FileStore is a very rudimentary way for storing characters in a similar way
// than the app already does. This is to avoid a lot of additional setup work
// for initial API development and to provide the simplest option for running
// a service for a small group of people.
// It stores all characters as local directories and revisions as files in those.
pub struct FileStore {
    root: PathBuf,

    characters: Characters,
}

type User = String;
type Uuid<'a> = &'a str;
type Revision = u64;

// TODO: switch to something like https://docs.rs/evmap/latest/evmap/
// Currently the CharacterMetadata is also gated by a lock to force sync access
// to single characters. This might not be practical, we'll see.
type Characters = RwLock<HashMap<String, Mutex<CharacterMetadata>>>;

impl FileStore {
    // new FileStore using the provided root directory
    pub fn new(root: PathBuf) -> Result<FileStore, Error> {
        let store = FileStore {
            root,
            characters: RwLock::new(HashMap::new()),
        };

        store.load_from_storage()?;

        Ok(store)
    }

    pub fn create_character(&self, owner: User) -> Result<String, Error> {
        trace!("creating character");

        let uuid = FileStore::new_uuid();

        let metadata = CharacterMetadata {
            uuid: uuid.clone(),
            owner,
            ..Default::default()
        };

        // unwrapping the write() here to panic on a poisoned lock
        let mut characters = self.characters.write().unwrap();

        match characters.entry(uuid.clone()) {
            Entry::Occupied(_) => {
                // this should never be able to happen as it means we generated a non unique UUID
                error!(uuid = uuid, "failed to create_character as the target UUID already exists");
                return Err(Error::new(
                    ErrorCode::Internal,
                    "failed to create new character, please retry",
                ));
            }
            Entry::Vacant(entry) => {
                metadata.create_character_directory(&self.root)?;
                metadata.write_to_root(&self.root)?;
                entry.insert(Mutex::new(metadata))
            }
        };

        Ok(uuid)
    }

    pub fn write_revision(
        &self,
        uuid: Uuid,
        user: User,
        character: Character,
        revision: Revision,
    ) -> Result<Revision, Error> {
        trace!("writing character revision");

        // unwrapping the read() here to panic on a poisoned lock
        let characters = self.characters.read().unwrap();

        let mut metadata = match characters.get(uuid) {
            Some(metadata) => metadata,
            None => {
                return Err(Error::new(
                    ErrorCode::NotFound,
                    "character does not exist",
                ));
            }
        }
        .lock()
        .unwrap();

        metadata.authorize(user)?;
        metadata.check_revision_order(revision)?;

        metadata.write_revision(&self.root, revision, character)?;

        metadata.update_latest_revision(&self.root, revision)?;

        Ok(revision)
    }

    pub fn read_revision(
        &self,
        uuid: Uuid,
        user: User,
        revision: Revision,
    ) -> Result<RevisionRead, Error> {
        trace!("reading character revision");

        // unwrapping the read() here to panic on a poisoned lock
        let characters = self.characters.read().unwrap();

        let metadata = match characters.get(uuid) {
            Some(metadata) => metadata,
            None => {
                return Err(Error::new(
                    ErrorCode::NotFound,
                    "character does not exist",
                ));
            }
        }
        .lock()
        .unwrap();

        metadata.authorize(user)?;

        let character = metadata.read_revision(&self.root, revision)?;

        Ok(RevisionRead {
            uuid: metadata.uuid.clone(),
            revision,
            character: SingularPtrField::some(character),
            ..Default::default()
        })
    }

    pub fn read_latest_revision(
        &self,
        uuid: Uuid,
        user: User,
    ) -> Result<RevisionRead, Error> {
        trace!("reading latest character revision");

        // unwrapping the read() here to panic on a poisoned lock
        let characters = self.characters.read().unwrap();

        let metadata = match characters.get(uuid) {
            Some(metadata) => metadata,
            None => {
                return Err(Error::new(
                    ErrorCode::NotFound,
                    "character does not exist",
                ));
            }
        }
        .lock()
        .unwrap();

        metadata.authorize(user)?;

        if !metadata.has_latest_revision() {
            return Err(Error::new(ErrorCode::NotFound, "failed to read latest revision for character without revisions"));
        }

        let character = metadata
            .read_revision(&self.root, metadata.get_latest_revision())?;

        Ok(RevisionRead {
            uuid: metadata.uuid.clone(),
            revision: metadata.get_latest_revision(),
            character: SingularPtrField::some(character),
            ..Default::default()
        })
    }

    fn load_from_storage(&self) -> Result<(), Error> {
        let paths = match fs::read_dir(&self.root) {
            Ok(paths) => paths,
            Err(err) => {
                error!(dir = ?self.root, err = %err, "failed to read characters from root");
                return Err(Error::new(
                    ErrorCode::Internal,
                    "failed to read characters from root",
                ));
            }
        };

        let mut characters = self.characters.write().unwrap();

        for path in paths {
            let path = match path {
                Ok(path) => path,
                Err(err) => {
                    warn!(err = %err, "failed to process path");
                    continue;
                }
            };
            if !path.metadata().unwrap().is_dir() {
                continue;
            }

            let path = path.file_name();
            let uuid = match path.to_str() {
                Some(uuid) => uuid,
                None => {
                    warn!(path = ?path, "failed to read uuid from path");
                    continue;
                }
            };

            let metadata = CharacterMetadata::read_from_root(&self.root, uuid)?;

            characters.insert(uuid.to_owned(), Mutex::new(metadata));
        }

        Ok(())
    }

    fn new_uuid() -> String {
        let mut buffer = uuid::Uuid::encode_buffer();
        uuid::Uuid::new_v4()
            .as_simple()
            .encode_lower(&mut buffer)
            .to_owned()
    }
}

// This block provides helpers on CharacterMetadata for accessing it in file storage.
// The functions defined here use the fields on CharacterMetadata as it's read
// from storage to avoid file access based on user input.
impl CharacterMetadata {
    pub fn read_from_root(
        root: &Path,
        uuid: Uuid,
    ) -> Result<CharacterMetadata, Error> {
        let path = CharacterMetadata::metadata_path(root, uuid);

        let mut file = match OpenOptions::new()
            .read(true)
            .write(false)
            .open(path)
        {
            Ok(file) => Ok(file),
            Err(err) => {
                error!( uuid = uuid, err = %err, "failed to open metadata file");
                match err.kind() {
                    std::io::ErrorKind::NotFound => Err(Error::new(
                        ErrorCode::NotFound,
                        "metadata file doesn't exists",
                    )),
                    _ => Err(Error::new(
                        ErrorCode::Internal,
                        "unexpected error opening revision",
                    )),
                }
            }
        }?;

        let metadata = match CharacterMetadata::parse_from_reader(&mut file) {
            Ok(metadata) => metadata,
            Err(err) => {
                error!( uuid = uuid, err = %err, "failed to decode metadata file");
                return Err(Error::new(
                    ErrorCode::Internal,
                    "could not decode metadata",
                ));
            }
        };

        Ok(metadata)
    }

    pub fn write_to_root(&self, root: &Path) -> Result<(), Error> {
        let path = CharacterMetadata::metadata_path(root, &self.uuid);

        self.write_to_file(
            OpenOptions::new().read(true).write(true).create_new(true),
            &path,
        )?;

        Ok(())
    }

    fn metadata_path(root: &Path, uuid: Uuid) -> PathBuf {
        let path = CharacterMetadata::character_path(root, uuid);
        path.join("metadata")
    }

    fn update_at_root(&self, root: &Path) -> Result<(), Error> {
        let path = CharacterMetadata::character_path(root, &self.uuid);
        let path = path.join("metadata");

        self.write_to_file(
            OpenOptions::new().read(true).write(true).truncate(true),
            &path,
        )?;

        Ok(())
    }

    fn write_to_file(
        &self,
        open: &mut OpenOptions,
        path: &Path,
    ) -> Result<(), Error> {
        let mut file = match open.open(path) {
            Ok(file) => Ok(file),
            Err(err) => {
                error!( path = ?path, err = %err, "failed to open metadata file");
                match err.kind() {
                    std::io::ErrorKind::AlreadyExists => Err(Error::new(
                        ErrorCode::Exists,
                        "metadata file already exists",
                    )),
                    std::io::ErrorKind::NotFound => Err(Error::new(
                        ErrorCode::NotFound,
                        "metadata file doesn't exists",
                    )),
                    _ => Err(Error::new(
                        ErrorCode::Internal,
                        "unexpected error writing metadata",
                    )),
                }
            }
        }?;

        let bytes = match self.write_to_bytes() {
            Ok(bytes) => bytes,
            Err(err) => {
                error!(uuid = self.uuid, err = %err, "failed to encode metadata");
                return Err(Error::new(
                    ErrorCode::Internal,
                    "failed to encode metadata",
                ));
            }
        };
        match file.write_all(&bytes) {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(uuid = self.uuid, err = %err, "failed to write metadata");
                return Err(Error::new(
                    ErrorCode::Internal,
                    "failed to write metadata",
                ));
            }
        }?;

        Ok(())
    }

    pub fn authorize(&self, user: User) -> Result<(), Error> {
        if user != self.owner {
            return Err(Error::new(ErrorCode::Unauthorized, "unauthorized"));
        }
        Ok(())
    }

    pub fn check_revision_order(
        &self,
        new_revision: Revision,
    ) -> Result<(), Error> {
        let latest_revision = self.get_latest_revision();
        let is_first_revision =
            new_revision == 0 && !self.has_latest_revision();
        if new_revision <= latest_revision && !is_first_revision {
            return Err(Error::new(
                ErrorCode::OutOfOrder,
                &format!("revisions need to be sent in order, last known revision is {latest_revision}"),
            ));
        }
        Ok(())
    }

    pub fn write_revision(
        &self,
        root: &Path,
        revision: Revision,
        character: Character,
    ) -> Result<Revision, Error> {
        let revision_file = self.create_revision_file(root, revision)?;
        self.write_revision_file(revision_file, character)?;

        Ok(revision)
    }

    pub fn update_latest_revision(
        &mut self,
        root: &Path,
        revision: Revision,
    ) -> Result<(), Error> {
        self.set_latest_revision(revision);

        self.update_at_root(root)?;

        Ok(())
    }

    pub fn read_revision(
        &self,
        root: &Path,
        revision: Revision,
    ) -> Result<Character, Error> {
        let mut revision_file = self.open_revision_file(root, revision)?;

        let character = match Character::parse_from_reader(&mut revision_file) {
            Ok(character) => character,
            Err(err) => {
                error!(uuid = self.uuid, revision = revision, err = %err, "failed to decode character revision");
                return Err(Error::new(
                    ErrorCode::Internal,
                    "could not decode character",
                ));
            }
        };

        Ok(character)
    }

    fn write_revision_file(
        &self,
        mut file: File,
        character: Character,
    ) -> Result<(), Error> {
        let bytes = match character.write_to_bytes() {
            Ok(bytes) => bytes,
            Err(err) => {
                error!(uuid = self.uuid, err = %err, "failed to encode character revision");
                return Err(Error::new(
                    ErrorCode::Internal,
                    "failed to encode character revision",
                ));
            }
        };
        match file.write_all(&bytes) {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(uuid = self.uuid, err = %err, "failed to write character revision");
                Err(Error::new(
                    ErrorCode::Internal,
                    "failed to write character revision",
                ))
            }
        }
    }

    fn create_revision_file(
        &self,
        root: &Path,
        revision: Revision,
    ) -> Result<File, Error> {
        trace!(
            uuid = &self.uuid,
            revision = revision,
            "creating character revision file"
        );
        let path = CharacterMetadata::revision_path(root, &self.uuid, revision);
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => Ok(file),
            Err(err) => {
                error!(uuid = self.uuid, revision = revision, path = ?path, err = %err, "failed to create revision file");
                match err.kind() {
                    std::io::ErrorKind::AlreadyExists => Err(Error::new(
                        ErrorCode::Exists,
                        &format!("revision {revision} already exists"),
                    )),
                    _ => Err(Error::new(
                        ErrorCode::Internal,
                        "unexpected error writing revision",
                    )),
                }
            }
        }
    }

    fn open_revision_file(
        &self,
        root: &Path,
        revision: Revision,
    ) -> Result<File, Error> {
        trace!(
            uuid = self.uuid,
            revision = revision,
            "opening character revision file"
        );
        let path = CharacterMetadata::revision_path(root, &self.uuid, revision);
        match OpenOptions::new().read(true).write(false).open(&path) {
            Ok(file) => Ok(file),
            Err(err) => {
                error!(uuid = self.uuid, revision = revision, path = ?path, err = %err, "failed to open revision file");
                match err.kind() {
                    std::io::ErrorKind::NotFound => Err(Error::new(
                        ErrorCode::NotFound,
                        &format!("revision {revision} doesn't exists"),
                    )),
                    _ => Err(Error::new(
                        ErrorCode::Internal,
                        "unexpected error opening revision",
                    )),
                }
            }
        }
    }

    fn revision_path(root: &Path, uuid: Uuid, revision: Revision) -> PathBuf {
        CharacterMetadata::character_path(root, uuid).join(revision.to_string())
    }

    fn create_character_directory(&self, root: &Path) -> Result<(), Error> {
        let path = CharacterMetadata::character_path(root, &self.uuid);
        trace!("creating character directory {}", path.display());
        match create_dir_all(path) {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(uuid = self.uuid, err = %err, "failed to create character directory");
                Err(Error::new(
                    ErrorCode::Internal,
                    "failed to create character",
                ))
            }
        }
    }

    fn character_path(root: &Path, uuid: Uuid) -> PathBuf {
        root.join(uuid)
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;
    use tempfile::tempdir;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    use crate::{
        db::{errors, file::FileStore},
        proto::storage::CharacterMetadata,
    };

    use proto_rs::character::Character;

    fn enable_logs() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }

    #[rstest]
    fn new_file_store() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let s = FileStore::new(root_path.clone()).unwrap();
        assert_eq!(s.root, root_path);
    }

    #[rstest]
    fn create_character() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let s = FileStore::new(root_path).unwrap();
        let uuid = s.create_character("test_user".to_owned());
        assert!(!uuid.expect("should return a new uuid").is_empty());
    }

    #[rstest]
    fn write_revision() {
        enable_logs();
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let s = FileStore::new(root_path).unwrap();

        let uuid = s.create_character("test_user".to_owned()).unwrap();

        let res = s.write_revision(
            &uuid,
            "test_user".to_owned(),
            Default::default(),
            0,
        );
        assert_eq!(res.expect("should return 0 for first write"), 0);
    }

    #[rstest]
    fn write_revision_requires_created_character() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let s = FileStore::new(root_path).unwrap();
        let res = s.write_revision(
            "test_uuid",
            "test_user".to_owned(),
            Default::default(),
            0,
        );
        assert!(
            res.is_err(),
            "write should return error if the character was not created"
        );
        assert_eq!(res.unwrap_err().code(), errors::ErrorCode::NotFound);
    }

    #[rstest]
    fn write_revision_restricts_access_to_owner() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let s = FileStore::new(root_path).unwrap();
        let uuid = s.create_character("owner".to_owned()).unwrap();
        let res = s.write_revision(
            &uuid,
            "other_user".to_owned(),
            Default::default(),
            0,
        );
        assert!(
            res.is_err(),
            "write should return error if the writer is not owner"
        );
        assert_eq!(res.unwrap_err().code(), errors::ErrorCode::Unauthorized);
    }

    #[rstest]
    fn write_revision_enforces_increasing_revisions() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let s = FileStore::new(root_path).unwrap();

        let uuid = s.create_character("test_user".to_owned()).unwrap();

        let res = s.write_revision(
            &uuid,
            "test_user".to_owned(),
            Default::default(),
            0,
        );
        assert_eq!(res.expect("should return 0 for first write"), 0);

        let res = s.write_revision(
            &uuid,
            "test_user".to_owned(),
            Default::default(),
            1,
        );
        assert_eq!(res.expect("should return 1 for second second"), 1);

        let res = s.write_revision(
            &uuid,
            "test_user".to_owned(),
            Default::default(),
            1,
        );
        assert_eq!(
            res.expect_err("should return error for third write").code(),
            errors::ErrorCode::OutOfOrder,
        );

        let res = s.write_revision(
            &uuid,
            "test_user".to_owned(),
            Default::default(),
            0,
        );
        assert_eq!(
            res.expect_err("should return error for fourth write")
                .code(),
            errors::ErrorCode::OutOfOrder,
        );
    }

    #[rstest]
    fn create_revision_file_does_not_overwrite() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let metadata = CharacterMetadata {
            uuid: "test_uuid".to_owned(),
            ..Default::default()
        };

        metadata
            .create_character_directory(&root_path)
            .expect("should create character directory");

        let file = metadata.create_revision_file(&root_path, 0);
        assert!(
            file.expect("should create revision file")
                .sync_all()
                .is_ok(),
            "should be able to sync file"
        );

        let file = metadata.create_revision_file(&root_path, 0);
        assert_eq!(
            file.expect_err("should fail to overwrite existing file")
                .code(),
            errors::ErrorCode::Exists
        );
    }

    #[rstest]
    fn open_revision_file_does_not_create() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let metadata = CharacterMetadata {
            uuid: "test_uuid".to_owned(),
            ..Default::default()
        };

        metadata
            .create_character_directory(&root_path)
            .expect("should create character directory");

        let file = metadata.create_revision_file(&root_path, 0);
        assert!(
            file.expect("should create revision file")
                .sync_all()
                .is_ok(),
            "should be able to sync file"
        );

        // let's open another revision that does not exist
        let file = metadata.open_revision_file(&root_path, 1);
        assert_eq!(
            file.expect_err("should fail to open missing file").code(),
            errors::ErrorCode::NotFound
        );
    }

    #[rstest]
    fn open_revision_file_read_only() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let metadata = CharacterMetadata {
            uuid: "test_uuid".to_owned(),
            ..Default::default()
        };

        metadata
            .create_character_directory(&root_path)
            .expect("should create character directory");

        let file = metadata.create_revision_file(&root_path, 0);
        assert!(
            file.expect("should create revision file")
                .sync_all()
                .is_ok(),
            "should be able to sync file"
        );

        let file = metadata.open_revision_file(&root_path, 0);
        assert!(file.is_ok(), "should open revision file");

        assert_eq!(
            metadata
                .write_revision_file(
                    file.unwrap(),
                    Character {
                        uuid: "test_uuid".to_owned(),
                        ..Default::default()
                    }
                )
                .expect_err("should fail to write to opened file")
                .code(),
            errors::ErrorCode::Internal
        );
    }

    #[rstest]
    fn write_revision_roundtrip() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let character: Character = Default::default();

        let s = FileStore::new(root_path).unwrap();

        let uuid = s.create_character("test_user".to_owned()).unwrap();

        let res = s.write_revision(
            &uuid,
            "test_user".to_owned(),
            character.clone(),
            0,
        );
        assert_eq!(res.expect("should return 0 for first write"), 0);

        let res = s.read_revision(&uuid, "test_user".to_owned(), 0);
        assert_eq!(
            res.expect("should return revision 0").character.unwrap(),
            character
        );
    }

    #[rstest]
    fn read_revision_requires_created_character() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let s = FileStore::new(root_path).unwrap();
        let res = s.read_revision("test_uuid", "test_user".to_owned(), 0);
        assert!(
            res.is_err(),
            "read should return error if the character was not created"
        );
        assert_eq!(res.unwrap_err().code(), errors::ErrorCode::NotFound);
    }

    #[rstest]
    fn read_revision_restricts_access_to_owner() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let s = FileStore::new(root_path).unwrap();
        let uuid = s.create_character("owner".to_owned()).unwrap();
        let res =
            s.write_revision(&uuid, "owner".to_owned(), Default::default(), 0);
        assert_eq!(res.expect("should return 0 for first write"), 0);

        let res = s.read_revision(&uuid, "other_user".to_owned(), 0);
        assert!(
            res.is_err(),
            "read should return error if the reader is not owner"
        );
        assert_eq!(res.unwrap_err().code(), errors::ErrorCode::Unauthorized);
    }

    #[rstest]
    fn new_loads_existing_data() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let uuid: String;
        {
            let s = FileStore::new(root_path.clone()).unwrap();
            uuid = s.create_character("test_user".to_owned()).unwrap();

            let res = s.write_revision(
                &uuid,
                "test_user".to_owned(),
                Default::default(),
                0,
            );
            assert_eq!(res.expect("should create first revision"), 0);
        }

        {
            let s = FileStore::new(root_path.clone()).unwrap();

            let res = s.write_revision(
                &uuid,
                "test_user".to_owned(),
                Default::default(),
                1,
            );
            assert_eq!(res.expect("should create second revision"), 1);
        }

        {
            let s = FileStore::new(root_path.clone()).unwrap();

            let res = s.read_latest_revision(&uuid, "test_user".to_owned());
            assert_eq!(res.expect("should return latest revision").revision, 1);
        }

        {
            let s = FileStore::new(root_path).unwrap();

            let res = s.write_revision(
                &uuid,
                "test_user".to_owned(),
                Default::default(),
                0,
            );
            assert_eq!(
                res.expect_err("should fail to create out of order revision")
                    .code(),
                errors::ErrorCode::OutOfOrder
            );
        }
    }

    #[rstest]
    fn read_latest_revision_fails_on_fresh_character() {
        let root = tempdir().unwrap();
        let root_path = root.path().to_owned();

        let s = FileStore::new(root_path).unwrap();
        let uuid = s.create_character("test_user".to_owned()).unwrap();

        let res = s.read_latest_revision(&uuid, "test_user".to_owned());
        assert_eq!(
            res.as_ref().expect_err("should not return revision").code(),
            errors::ErrorCode::NotFound
        );
        assert_eq!(
            res.as_ref()
                .expect_err("should not return revision")
                .message(),
            "failed to read latest revision for character without revisions"
        );
    }
}
