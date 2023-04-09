// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_CHARACTERS_CREATE: ::grpcio::Method<super::characters::CreateCharacter, super::characters::CharacterCreated> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/characters.Characters/Create",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_CHARACTERS_WRITE_CHARACTER_REVISION: ::grpcio::Method<super::characters::WriteRevision, super::characters::RevisionWritten> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/characters.Characters/WriteCharacterRevision",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_CHARACTERS_READ_CHARACTER_REVISION: ::grpcio::Method<super::characters::ReadRevision, super::characters::RevisionRead> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/characters.Characters/ReadCharacterRevision",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
}; 

const METHOD_CHARACTERS_READ_LATEST_CHARACTER_REVISION: ::grpcio::Method<super::characters::ReadLatestRevision, super::characters::RevisionRead> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/characters.Characters/ReadLatestCharacterRevision",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_CHARACTERS_DELETE: ::grpcio::Method<super::characters::DeleteCharacter, super::characters::CharacterDeleted> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/characters.Characters/Delete",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct CharactersClient {
    client: ::grpcio::Client,
}

impl CharactersClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        CharactersClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn create_opt(&self, req: &super::characters::CreateCharacter, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::characters::CharacterCreated> {
        self.client.unary_call(&METHOD_CHARACTERS_CREATE, req, opt)
    }

    pub fn create(&self, req: &super::characters::CreateCharacter) -> ::grpcio::Result<super::characters::CharacterCreated> {
        self.create_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_async_opt(&self, req: &super::characters::CreateCharacter, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::CharacterCreated>> {
        self.client.unary_call_async(&METHOD_CHARACTERS_CREATE, req, opt)
    }

    pub fn create_async(&self, req: &super::characters::CreateCharacter) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::CharacterCreated>> {
        self.create_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn write_character_revision_opt(&self, req: &super::characters::WriteRevision, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::characters::RevisionWritten> {
        self.client.unary_call(&METHOD_CHARACTERS_WRITE_CHARACTER_REVISION, req, opt)
    }

    pub fn write_character_revision(&self, req: &super::characters::WriteRevision) -> ::grpcio::Result<super::characters::RevisionWritten> {
        self.write_character_revision_opt(req, ::grpcio::CallOption::default())
    }

    pub fn write_character_revision_async_opt(&self, req: &super::characters::WriteRevision, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::RevisionWritten>> {
        self.client.unary_call_async(&METHOD_CHARACTERS_WRITE_CHARACTER_REVISION, req, opt)
    }

    pub fn write_character_revision_async(&self, req: &super::characters::WriteRevision) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::RevisionWritten>> {
        self.write_character_revision_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn read_character_revision_opt(&self, req: &super::characters::ReadRevision, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::characters::RevisionRead> {
        self.client.unary_call(&METHOD_CHARACTERS_READ_CHARACTER_REVISION, req, opt)
    }

    pub fn read_character_revision(&self, req: &super::characters::ReadRevision) -> ::grpcio::Result<super::characters::RevisionRead> {
        self.read_character_revision_opt(req, ::grpcio::CallOption::default())
    }

    pub fn read_character_revision_async_opt(&self, req: &super::characters::ReadRevision, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::RevisionRead>> {
        self.client.unary_call_async(&METHOD_CHARACTERS_READ_CHARACTER_REVISION, req, opt)
    }

    pub fn read_character_revision_async(&self, req: &super::characters::ReadRevision) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::RevisionRead>> {
        self.read_character_revision_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn read_latest_character_revision_opt(&self, req: &super::characters::ReadLatestRevision, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::characters::RevisionRead> {
        self.client.unary_call(&METHOD_CHARACTERS_READ_LATEST_CHARACTER_REVISION, req, opt)
    }

    pub fn read_latest_character_revision(&self, req: &super::characters::ReadLatestRevision) -> ::grpcio::Result<super::characters::RevisionRead> {
        self.read_latest_character_revision_opt(req, ::grpcio::CallOption::default())
    }

    pub fn read_latest_character_revision_async_opt(&self, req: &super::characters::ReadLatestRevision, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::RevisionRead>> {
        self.client.unary_call_async(&METHOD_CHARACTERS_READ_LATEST_CHARACTER_REVISION, req, opt)
    }

    pub fn read_latest_character_revision_async(&self, req: &super::characters::ReadLatestRevision) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::RevisionRead>> {
        self.read_latest_character_revision_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_opt(&self, req: &super::characters::DeleteCharacter, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::characters::CharacterDeleted> {
        self.client.unary_call(&METHOD_CHARACTERS_DELETE, req, opt)
    }

    pub fn delete(&self, req: &super::characters::DeleteCharacter) -> ::grpcio::Result<super::characters::CharacterDeleted> {
        self.delete_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_async_opt(&self, req: &super::characters::DeleteCharacter, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::CharacterDeleted>> {
        self.client.unary_call_async(&METHOD_CHARACTERS_DELETE, req, opt)
    }

    pub fn delete_async(&self, req: &super::characters::DeleteCharacter) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::characters::CharacterDeleted>> {
        self.delete_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::std::future::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Characters {
    fn create(&mut self, ctx: ::grpcio::RpcContext, _req: super::characters::CreateCharacter, sink: ::grpcio::UnarySink<super::characters::CharacterCreated>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn write_character_revision(&mut self, ctx: ::grpcio::RpcContext, _req: super::characters::WriteRevision, sink: ::grpcio::UnarySink<super::characters::RevisionWritten>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn read_character_revision(&mut self, ctx: ::grpcio::RpcContext, _req: super::characters::ReadRevision, sink: ::grpcio::UnarySink<super::characters::RevisionRead>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn read_latest_character_revision(&mut self, ctx: ::grpcio::RpcContext, _req: super::characters::ReadLatestRevision, sink: ::grpcio::UnarySink<super::characters::RevisionRead>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn delete(&mut self, ctx: ::grpcio::RpcContext, _req: super::characters::DeleteCharacter, sink: ::grpcio::UnarySink<super::characters::CharacterDeleted>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_characters<S: Characters + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_CHARACTERS_CREATE, move |ctx, req, resp| {
        instance.create(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_CHARACTERS_WRITE_CHARACTER_REVISION, move |ctx, req, resp| {
        instance.write_character_revision(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_CHARACTERS_READ_CHARACTER_REVISION, move |ctx, req, resp| {
        instance.read_character_revision(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_CHARACTERS_READ_LATEST_CHARACTER_REVISION, move |ctx, req, resp| {
        instance.read_latest_character_revision(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_CHARACTERS_DELETE, move |ctx, req, resp| {
        instance.delete(ctx, req, resp)
    });
    builder.build()
}
