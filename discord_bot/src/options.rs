use clap::Args;
use env_arg::EnvArgs;
use env_arg_derive::EnvArgs;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Args, EnvArgs)]
pub struct Options {
    #[arg(long, env)]
    pub discord_token: String,
}
