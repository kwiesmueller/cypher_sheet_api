use std::{
    env::temp_dir, fs::File, io::Write, path::PathBuf, process::Command,
};

use tracing::{debug, info};

struct ContainerFile {
    image_layers: Vec<String>,
}

impl ContainerFile {
    fn new() -> ContainerFile {
        ContainerFile {
            image_layers: vec![],
        }
    }

    fn layer(&mut self, add: &str) {
        self.layer_string(add.to_owned());
    }

    fn layer_string(&mut self, add: String) {
        self.image_layers.push(add);
    }

    fn render(self) -> String {
        self.image_layers.join("\n\n")
    }

    fn use_build_image(&mut self, image: &str) {
        self.layer(&["FROM", image, "as builder"].join(" "));
    }

    fn install_cmake(&mut self) {
        self.layer("RUN apt-get update && apt-get install -y cmake && rm -rf /var/lib/apt/lists/*");
    }

    fn set_workdir(&mut self, workdir: &str) {
        self.layer(&["WORKDIR", workdir].join(" "));
    }

    fn set_env(&mut self, key: &str, val: &str) {
        self.layer_string(format!("ENV {key}={val}"));
    }

    fn copy(&mut self, src: &str, dst: &str) {
        self.layer(&["COPY", src, dst].join(" "));
    }

    fn prepare_dependency_build(&mut self, component: &str) {
        let noop_main_file = "fn main() {}";

        self.layer_string(format!(
            r###"
            COPY {component}/Cargo.toml Cargo.lock /usr/src/{component}/
            RUN \
                mkdir /usr/src/{component}/src && \
                echo '{noop_main_file}' > /usr/src/{component}/src/main.rs"###
        ));
    }

    fn run_cargo_build(&mut self, component: &str) {
        self.layer_string(format!(
            "RUN cargo build --release --bin {component}"
        ));
    }

    fn copy_sources(&mut self, component: &str) {
        self.layer_string(format!(
            "COPY {component}/src /usr/src/{component}/src/"
        ));
    }

    fn touch_main_file(&mut self, component: &str) {
        self.layer_string(format!(
            "RUN touch /usr/src/{component}/src/main.rs"
        ));
    }

    fn persist_build_artifact(&mut self, component: &str) {
        self.layer_string(format!(
            "RUN cp /usr/src/{component}/target/release/{component} /"
        ));
    }

    fn use_runtime_image(&mut self, image: &str) {
        self.layer(&["FROM", image, "as runtime"].join(" "));
    }

    fn copy_build_artifact_to_runtime(&mut self, component: &str) {
        self.layer_string(format!("COPY --from=builder /{component} /"));
    }

    fn set_cmd(&mut self, cmd: &str) {
        self.layer_string(format!(r##"CMD ["/{cmd}"]"##));
    }
}

fn container_file(
    component: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut image = ContainerFile::new();

    let build_image = "docker.io/rust:1.68-bullseye";
    let runtime_image = "docker.io/debian:bullseye-slim";

    image.use_build_image(build_image);
    image.install_cmake();

    let workdir = "/usr/src";
    image.set_workdir(workdir);
    image.set_env("CARGO_REGISTRIES_CRATES_IO_PROTOCOL", "sparse");

    image.copy("proto_rs", "/usr/src/proto_rs");

    image.prepare_dependency_build(component);

    let workdir = &["/usr/src/", component].concat();
    image.set_workdir(workdir);

    // Build the dependencies (only).
    image.run_cargo_build(component);

    image.copy_sources(component);
    image.touch_main_file(component);

    // Actually build the binary this time.
    image.run_cargo_build(component);
    image.persist_build_artifact(component);

    // Build final container.
    image.use_runtime_image(runtime_image);
    image.copy_build_artifact_to_runtime(component);
    image.set_cmd(component);

    let rendered_container_file = image.render();

    let temp = temp_dir();
    let container_file_path = temp.join("ContainerFile");
    let mut container_file = File::create(container_file_path.clone())?;
    container_file.write_all(rendered_container_file.as_bytes())?;

    Ok(container_file_path)
}

pub async fn image(
    component: &str,
    image_name: &str,
    image_tag: &str,
    cargo_registry_cache: Option<String>,
    target_cache: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let container_file_path = container_file(component)?;
    let container_file_path = container_file_path
        .to_str()
        .expect("should create a valid Containerfile path");

    build_image(
        container_file_path,
        image_name,
        image_tag,
        cargo_registry_cache,
        target_cache,
    )?;
    push_image(image_name, image_tag)?;

    Ok(())
}

pub fn build_image(
    container_file_path: &str,
    image_name: &str,
    image_tag: &str,
    cargo_registry_cache: Option<String>,
    target_cache: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("building image \"{image_name}:{image_tag}\"");

    let mut cmd = Command::new("buildah");

    cmd.arg("build")
        .arg(format!("--file={container_file_path}"));

    if let Some(cargo_registry_cache) = cargo_registry_cache {
        cmd.arg(format!(
            "--volume={cargo_registry_cache}:/usr/local/cargo/registry:O"
        ));
    }

    if let Some(target_cache) = target_cache {
        cmd.arg(format!(
            "--volume={target_cache}:/usr/src/discord_bot/target:U,Z"
        ));
    }

    cmd.arg(format!("--tag={image_name}:{image_tag}"));
    cmd.arg("--layers").arg(".");

    let status = cmd.status()?;
    info!(
        "finished building image {image_name}:{image_tag} with status {status}"
    );

    Ok(())
}

pub fn push_image(
    image_name: &str,
    image_tag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("pushing image \"{image_name}:{image_tag}\"");

    let mut cmd = Command::new("buildah");
    cmd.arg("push").arg(format!("{image_name}:{image_tag}"));

    let status = cmd.status()?;
    info!(
        "finished pushing image {image_name}:{image_tag} with status {status}"
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::ContainerFile;

    #[rstest]
    fn container_file_renders() {
        let mut image = ContainerFile::new();
        image.layer("FROM scratch");
        image.layer("RUN make build");

        let expect = r###"FROM scratch

RUN make build"###;

        assert_eq!(image.render(), expect)
    }
}
