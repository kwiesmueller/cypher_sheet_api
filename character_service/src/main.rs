mod proto;

mod db;

use std::{
    io::{self, Read},
    sync::Arc,
    thread,
};

use futures::{channel::oneshot, executor::block_on};
use grpcio::{
    ChannelBuilder, Environment, ResourceQuota, RpcStatus, RpcStatusCode,
    ServerBuilder, ServerCredentials,
};
use proto_rs::{
    characters::{
        ReadLatestRevision, ReadRevision, RevisionRead, WriteRevision,
    },
    characters_grpc::{self},
};
use tracing::{error, info, span, trace, Level};
use tracing_subscriber::FmtSubscriber;

use proto_rs::{
    characters::{CharacterCreated, CreateCharacter, RevisionWritten},
    characters_grpc::create_characters,
};

#[derive(Clone)]
struct CharacterService {
    db: Arc<db::FileStore>,
}

impl characters_grpc::Characters for CharacterService {
    fn create(
        &mut self,
        _ctx: grpcio::RpcContext,
        _req: CreateCharacter,
        sink: grpcio::UnarySink<CharacterCreated>,
    ) {
        let span = span!(
            target: "character_service",
            Level::TRACE,
            "write_character_revision",
        );
        let _enter = span.enter();

        // TODO: for the hosted service we need to add authn/authz so characters are
        // gated to users.
        // It doesn't serve much benefit to add right now as it would only be
        // access token based and the reason to have the API in the first place
        // is as a backup. If someone looses their phone they also probably
        // lose the access token. So uuids need to be enough for now, that's
        // already meh. For the hosted service things need to be tied to email or
        // social accounts.

        trace!(parent: &span, "received character creation request",);

        match self.db.clone().create_character("test_user".to_owned()) {
            Ok(uuid) => sink.success(CharacterCreated {
                uuid,
                ..Default::default()
            }),
            Err(err) => sink.fail(err.into()),
        };
    }

    fn write_character_revision(
        &mut self,
        _ctx: grpcio::RpcContext,
        mut req: WriteRevision,
        sink: grpcio::UnarySink<RevisionWritten>,
    ) {
        let span = span!(
            target: "character_service",
            Level::TRACE,
            "write_character_revision",
        );
        let _enter = span.enter();

        // TODO: for the hosted service we need to add authn/authz so characters are
        // gated to users.
        // It doesn't serve much benefit to add right now as it would only be
        // access token based and the reason to have the API in the first place
        // is as a backup. If someone looses their phone they also probably
        // lose the access token. So uuids need to be enough for now, that's
        // already meh. For the hosted service things need to be tied to email or
        // social accounts.

        let character = match req.character.take() {
            Some(character) => character,
            None => {
                error!(parent: &span, "missing character data");
                sink.fail(RpcStatus::with_message(
                    RpcStatusCode::INVALID_ARGUMENT,
                    "missing character data".to_owned(),
                ));
                return;
            }
        };

        let character_uuid = character.uuid.clone();

        trace!(
            parent: &span,
            character = character_uuid,
            revision = req.revision,
            "received character revision",
        );

        match self.db.clone().write_revision(
            &character_uuid,
            "test_user".to_owned(),
            character,
            req.revision,
        ) {
            Ok(revision) => sink.success(RevisionWritten {
                revision,
                uuid: character_uuid,
                ..Default::default()
            }),
            Err(err) => sink.fail(err.into()),
        };
    }

    fn read_character_revision(
        &mut self,
        _ctx: grpcio::RpcContext,
        req: ReadRevision,
        sink: grpcio::UnarySink<RevisionRead>,
    ) {
        let span = span!(
            target: "character_service",
            Level::TRACE,
            "read_character_revision",
        );
        let _enter = span.enter();

        // TODO: for the hosted service we need to add authn/authz so characters are
        // gated to users.
        // It doesn't serve much benefit to add right now as it would only be
        // access token based and the reason to have the API in the first place
        // is as a backup. If someone looses their phone they also probably
        // lose the access token. So uuids need to be enough for now, that's
        // already meh. For the hosted service things need to be tied to email or
        // social accounts.

        trace!(
            parent: &span,
            uuid = req.uuid,
            revision = req.revision,
            "received character read request",
        );

        match self.db.clone().read_revision(
            &req.uuid,
            "test_user".to_owned(),
            req.revision,
        ) {
            Ok(revision) => sink.success(revision),
            Err(err) => sink.fail(err.into()),
        };
    }

    fn read_latest_character_revision(
        &mut self,
        _ctx: grpcio::RpcContext,
        req: ReadLatestRevision,
        sink: grpcio::UnarySink<RevisionRead>,
    ) {
        let span = span!(
            target: "character_service",
            Level::TRACE,
            "read_character_revision",
        );
        let _enter = span.enter();

        // TODO: for the hosted service we need to add authn/authz so characters are
        // gated to users.
        // It doesn't serve much benefit to add right now as it would only be
        // access token based and the reason to have the API in the first place
        // is as a backup. If someone looses their phone they also probably
        // lose the access token. So uuids need to be enough for now, that's
        // already meh. For the hosted service things need to be tied to email or
        // social accounts.

        trace!(
            parent: &span,
            uuid = req.uuid,
            "received character read latest request",
        );

        match self
            .db
            .clone()
            .read_latest_revision(&req.uuid, "test_user".to_owned())
        {
            Ok(revision) => sink.success(revision),
            Err(err) => sink.fail(err.into()),
        };
    }
}

fn new_file_store() -> Arc<db::FileStore> {
    Arc::new(db::FileStore::new("./testdata/".to_owned().into()).unwrap())
}

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let db = new_file_store();

    let service = create_characters(CharacterService { db });

    let env = Arc::new(Environment::new(1));
    let addr = "127.0.0.1:8080";
    let quota = ResourceQuota::new(Some("CharacterServiceQuota"))
        .resize_memory(1024 * 1024);
    let ch_builder = ChannelBuilder::new(env.clone()).set_resource_quota(quota);

    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .channel_args(ch_builder.build_args())
        .build()
        .unwrap();
    server
        .add_listening_port(addr, ServerCredentials::insecure())
        .unwrap();
    server.start();
    info!("listening on {addr}");
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        info!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    let _ = block_on(rx);
    let _ = block_on(server.shutdown());
}

#[cfg(test)]
mod tests {}
