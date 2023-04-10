use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{Namespace, Secret},
};
use kube::{
    api::{Patch, PatchParams},
    Api, Client,
};

use crate::{components::Component, resources::expected_secret};

use super::{
    options::Environment,
    resources::{
        default_labels, expected_deployment, expected_namespace, FIELD_MANAGER,
        PROJECT,
    },
};

pub async fn deploy(
    env: Environment,
    component: &Component,
    image_registry: &str,
    tag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let _labels = default_labels(env, component);
    reconcile_namespace(&client, env).await?;
    reconcile_secret(&client, env, component).await?;
    reconcile_deployment(&client, env, component, image_registry, tag).await?;

    Ok(())
}

async fn reconcile_namespace(
    client: &Client,
    env: Environment,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch: Namespace = expected_namespace(PROJECT, env);

    let params = PatchParams::apply(FIELD_MANAGER).force();
    let namespaces: Api<Namespace> = Api::all(client.to_owned());
    namespaces
        .patch(PROJECT, &params, &Patch::Apply(&patch))
        .await?;

    Ok(())
}

async fn reconcile_deployment(
    client: &Client,
    env: Environment,
    component: &Component,
    image_registry: &str,
    tag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch: Deployment =
        expected_deployment(env, image_registry, component, tag);

    let params = PatchParams::apply(FIELD_MANAGER).force();
    let deployments: Api<Deployment> =
        Api::namespaced(client.to_owned(), PROJECT);
    deployments
        .patch(component.resource_name(), &params, &Patch::Apply(&patch))
        .await?;

    Ok(())
}

async fn reconcile_secret(
    client: &Client,
    env: Environment,
    component: &Component,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch: Secret = expected_secret(env, component);

    let params = PatchParams::apply(FIELD_MANAGER).force();
    let secrets: Api<Secret> = Api::namespaced(client.to_owned(), PROJECT);
    secrets
        .patch(component.resource_name(), &params, &Patch::Apply(&patch))
        .await?;

    Ok(())
}
