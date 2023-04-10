use std::collections::BTreeMap;

pub trait EnvArgs {
    fn get_env_pairs(&self) -> BTreeMap<String, String>;
}
