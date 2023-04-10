use clap::Parser;
use discord_bot::Options;
use dotenv::dotenv;
use embeds::Embedable;
use proto_rs::character::{SharedObject, SharedObject_oneof_object};
use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    json::{self, Value},
    model::prelude::MessageReference,
    prelude::{EventHandler, GatewayIntents},
    Client,
};

use tracing::{debug, error, Level};
use tracing_subscriber::FmtSubscriber;

use protobuf::Message;

mod embeds;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(
        &self,
        ctx: serenity::prelude::Context,
        msg: serenity::model::prelude::Message,
    ) {
        if msg.attachments.is_empty() {
            return;
        }

        for attachment in &msg.attachments {
            let raw = match attachment.download().await {
                Ok(content) => content,
                Err(err) => {
                    error!("failed to download file: {err}");
                    continue;
                }
            };

            let shared_object = match SharedObject::parse_from_bytes(&raw) {
                Ok(shared_object) => shared_object,
                Err(err) => {
                    debug!(
                        "failed to decode attachment into SharedObject: {err}"
                    );
                    continue;
                }
            };

            let mut embed = match shared_object_to_embed(shared_object) {
                Some(embed) => embed,
                None => {
                    continue;
                }
            };

            embed.url(&attachment.url);

            let mut new_message = CreateMessage::default();
            new_message.set_embed(embed);
            new_message.reference_message(MessageReference::from(&msg));

            // TODO: switch to the next version of serenity to make this nicer.
            let new_message = json::hashmap_to_json_map(new_message.0);
            match ctx
                .http
                .send_message(msg.channel_id.0, &Value::from(new_message))
                .await
            {
                Ok(_) => {
                    debug!("sent message and received response");
                }
                Err(err) => {
                    error!("failed to send message: {err}");
                }
            };
        }
    }
}

fn shared_object_to_embed(obj: SharedObject) -> Option<CreateEmbed> {
    let inner = match obj.object {
        Some(inner) => inner,
        None => {
            error!("received SharedObject without inner object");
            return None;
        }
    };

    match inner {
        SharedObject_oneof_object::character(_) => {
            // Characters are unsupported right now.
            None
        }
        SharedObject_oneof_object::skill(skill) => Some(skill.embed()),
        SharedObject_oneof_object::ability(ability) => Some(ability.embed()),
        SharedObject_oneof_object::cypher(cypher) => Some(cypher.embed()),
        SharedObject_oneof_object::artifact(artifact) => Some(artifact.embed()),
        SharedObject_oneof_object::item(item) => Some(item.embed()),
        SharedObject_oneof_object::note(note) => Some(note.embed()),
    }
}

#[derive(Parser)]
struct Command {
    #[command(flatten)]
    options: Options,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cmd = Command::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // Build our client.
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(cmd.options.discord_token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
