use std::{error::Error, fmt};

use assert_json_diff::{assert_json_matches_no_panic, CompareMode};

use k8s_openapi::{
    api::{
        apps::v1::Deployment,
        core::v1::{Namespace, Secret},
    },
    serde::{de::DeserializeOwned, Serialize},
};
use kube::{Api, Client, ResourceExt};

use tracing::{error, info};

use crate::{components::Component, resources::expected_secret};

use super::{
    options::Environment,
    resources::{expected_deployment, expected_namespace, PROJECT},
};

pub async fn status(
    env: Environment,
    component: &Component,
    image_registry: &str,
    tag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;

    check_namespace(client.clone(), env).await?;
    check_secret(client.clone(), env, component).await?;
    check_deployment(client.clone(), env, component, image_registry, tag)
        .await?;

    Ok(())
}

async fn check_namespace(
    client: Client,
    env: Environment,
) -> Result<(), Box<dyn std::error::Error>> {
    check_resource::<Namespace>(
        Api::all(client),
        expected_namespace(PROJECT, env),
    )
    .await
}

async fn check_deployment(
    client: Client,
    env: Environment,
    component: &Component,
    image_registry: &str,
    tag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let expect = expected_deployment(env, image_registry, component, tag);

    let deployment =
        get_resource(Api::namespaced(client, PROJECT), expect.clone()).await?;
    check_resource_against::<Deployment>(expect, deployment.clone())?;

    check_deployment_ready_pods(&deployment, 1)?;

    Ok(())
}

async fn check_secret(
    client: Client,
    env: Environment,
    component: &Component,
) -> Result<(), Box<dyn std::error::Error>> {
    let expect = expected_secret(env, component);

    check_resource::<Secret>(Api::namespaced(client, PROJECT), expect).await
}

#[derive(Debug)]
struct ResourceDiff {
    kind: String,
    name: String,
    diff: String,
}

impl ResourceDiff {
    fn new(kind: String, name: String, diff: String) -> ResourceDiff {
        ResourceDiff { kind, name, diff }
    }

    fn new_boxed(
        kind: String,
        name: String,
        diff: String,
    ) -> Box<ResourceDiff> {
        Box::new(ResourceDiff::new(kind, name, diff))
    }
}

impl Error for ResourceDiff {}

impl fmt::Display for ResourceDiff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} does not match expectations:\n{}",
            self.kind, self.name, self.diff
        )
    }
}

async fn check_resource<Resource>(
    client: Api<Resource>,
    expect: Resource,
) -> Result<(), Box<dyn std::error::Error>>
where
    Resource: Serialize,
    Resource: Clone,
    Resource: DeserializeOwned,
    Resource: std::fmt::Debug,
    Resource: k8s_openapi::Resource,
    Resource: kube::Resource,
{
    let kind = k8s_openapi::kind(&expect);
    let name = expect.name_any();

    let resource = match get_resource(client, expect.clone()).await {
        Ok(resource) => resource,
        Err(err) => {
            error!("Failed to read {kind} {name}: {err}");
            return Err(err);
        }
    };
    check_resource_against(expect, resource)
}

fn check_resource_against<Resource>(
    expect: Resource,
    got: Resource,
) -> Result<(), Box<dyn std::error::Error>>
where
    Resource: Serialize,
    Resource: Clone,
    Resource: DeserializeOwned,
    Resource: std::fmt::Debug,
    Resource: k8s_openapi::Resource,
    Resource: kube::Resource,
{
    let kind = k8s_openapi::kind(&expect);
    let name = expect.name_any();

    info!("{kind} {name} exists");
    let diff = diff_resource(&got, &expect);
    if let Some(diff) = diff {
        error!("{kind} {name} does not match expectations:\n{diff}");
        return Err(ResourceDiff::new_boxed(kind.to_owned(), name, diff));
    } else {
        info!("{kind} {name} is ok");
    }

    Ok(())
}

async fn get_resource<Resource>(
    client: Api<Resource>,
    expect: Resource,
) -> Result<Resource, Box<dyn std::error::Error>>
where
    Resource: Serialize,
    Resource: Clone,
    Resource: DeserializeOwned,
    Resource: std::fmt::Debug,
    Resource: k8s_openapi::Resource,
    Resource: kube::Resource,
{
    let name = expect.name_any();

    Ok(client.get(&name).await?)
}

fn diff_resource<Got, Want>(got: &Got, want: &Want) -> Option<String>
where
    Got: Serialize,
    Want: Serialize,
{
    let diff_config = assert_json_diff::Config::new(CompareMode::Inclusive);
    assert_json_matches_no_panic(got, want, diff_config).err()
}

fn check_deployment_ready_pods(
    deployment: &Deployment,
    min_ready: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let kind = k8s_openapi::kind(deployment);
    let name = deployment.name_any();
    match &deployment.status {
        Some(status) => match status.ready_replicas {
            Some(ready) => {
                info!("{ready} pods are ready");
                if ready < min_ready {
                    return Err(Box::new(ResourceDiff::new(
                        kind.to_owned(),
                        name,
                        "not enough ready pods".to_owned(),
                    )));
                }
            }
            None => {
                return Err(Box::new(ResourceDiff::new(
                    kind.to_owned(),
                    name,
                    "no ready pods".to_owned(),
                )))
            }
        },
        None => {
            return Err(Box::new(ResourceDiff::new(
                kind.to_owned(),
                name,
                "no status".to_owned(),
            )))
        }
    };

    Ok(())
}
