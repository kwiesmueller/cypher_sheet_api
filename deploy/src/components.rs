use std::collections::BTreeMap;

use clap::Subcommand;

use env_arg::EnvArgs;

#[derive(Subcommand, Clone)]
pub enum Component {
    DiscordBot {
        #[command(flatten)]
        options: discord_bot::Options,
    },
}

impl Component {
    pub fn name(&self) -> &str {
        self.spec().name
    }

    pub fn image(&self, registry: &str, tag: &str) -> String {
        format!("{registry}/{}:{tag}", self.image_name())
    }

    pub fn image_name(&self) -> &str {
        self.spec().image_name
    }

    pub fn resource_name(&self) -> &str {
        self.spec().resource_name
    }

    pub fn environment_variables(&self) -> BTreeMap<String, String> {
        self.spec().environment_variables
    }

    fn spec(&self) -> ComponentSpec {
        match self {
            Component::DiscordBot { options } => discord_bot_spec(options),
        }
    }
}

pub struct ComponentSpec<'a> {
    name: &'a str,
    image_name: &'a str,
    resource_name: &'a str,
    environment_variables: BTreeMap<String, String>,
}

fn discord_bot_spec(options: &discord_bot::Options) -> ComponentSpec<'static> {
    let environment_variables = options.get_env_pairs();

    ComponentSpec {
        name: "discord_bot",
        image_name: "cypher-sheet-discord-bot",
        resource_name: "discord-bot",
        environment_variables,
    }
}
