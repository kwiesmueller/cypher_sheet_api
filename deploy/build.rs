use std::{env, path};

use built::Options;

fn main() {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    built::write_built_file_with_opts(
        Options::default().set_git(true),
        src.as_ref(),
        &dst,
    )
    .expect("Failed to acquire build-time information");
}
