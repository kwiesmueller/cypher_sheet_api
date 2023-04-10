use clap::{Parser, Subcommand, ValueEnum};
use std::ffi::OsString;
use std::fmt::Debug;
use std::fmt::Display;

use std::path::PathBuf;

use crate::components::Component;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Parser)]
pub struct Options {
    #[arg(value_enum, short, long, env, default_value_t = Environment::Test)]
    pub env: Environment,

    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, env, default_value = "quay.io/kwiesmueller")]
    pub image_registry: String,

    #[arg(long, env, default_value = built_info::GIT_VERSION.unwrap_or("unknown"))]
    pub image_tag: String,
}

#[derive(Subcommand)]
pub enum Commands {
    Status {
        #[command(subcommand)]
        component: Component,
    },
    Deploy {
        #[command(subcommand)]
        component: Component,
    },
    Image {
        #[command(subcommand)]
        component: Component,

        #[arg(long, env, default_value = path_in_home_dir(".cargo/registry"))]
        cargo_registry_cache: Option<String>,
        #[arg(long, env, default_value = "./target_buildah")]
        target_cache: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Environment {
    Test,
    Staging,
    Prod,
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Environment::Test => "Test",
            Environment::Staging => "Staging",
            Environment::Prod => "Prod",
        })
    }
}

fn home_dir() -> PathBuf {
    let home = std::env::var_os("HOME").unwrap_or(OsString::from("/tmp/"));
    PathBuf::from(home)
}

fn path_in_home_dir(path: &'static str) -> OsString {
    home_dir().join(path).into_os_string()
}
