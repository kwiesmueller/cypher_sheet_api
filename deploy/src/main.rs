use clap::{error::ErrorKind, CommandFactory, Parser};
use dotenv::dotenv;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use self::{
    deploy::deploy,
    image::image,
    options::{Commands, Options},
    status::status,
};

mod components;
mod deploy;
mod image;
mod options;
mod resources;
mod status;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let options = Options::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    if let Some(Commands::Deploy { component }) = options.command {
        return deploy(
            options.env,
            &component,
            &options.image_registry,
            &options.image_tag,
        )
        .await;
    }

    if let Some(Commands::Image {
        component,
        cargo_registry_cache,
        target_cache,
    }) = options.command
    {
        return image(
            component.name(),
            &component.image(&options.image_registry, &options.image_tag),
            &options.image_tag,
            cargo_registry_cache,
            target_cache,
        )
        .await;
    }

    if let Some(Commands::Status { component }) = options.command {
        return status(
            options.env,
            &component,
            &options.image_registry,
            &options.image_tag,
        )
        .await;
    }

    Err(Box::new(Options::command().error(
        ErrorKind::MissingSubcommand,
        "missing required command",
    )))
}
