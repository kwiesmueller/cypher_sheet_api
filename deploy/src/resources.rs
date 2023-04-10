use std::{collections::BTreeMap, vec};

use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Container, EnvVar, EnvVarSource, Namespace, PodSpec,
            PodTemplateSpec, Secret, SecretKeySelector,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
    ByteString,
};
use kube::core::ObjectMeta;

use crate::components::Component;

use super::options::{built_info, Environment};

pub const PROJECT: &str = "cypher-sheet";
pub const FIELD_MANAGER: &str = "cypher-sheet-deploy";

pub fn expected_namespace(name: &str, env: Environment) -> Namespace {
    let labels = default_namespace_labels(env);
    Namespace {
        metadata: ObjectMeta {
            name: Some(name.to_owned()),
            labels: Some(labels),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn expected_deployment(
    env: Environment,
    image_registry: &str,
    component: &Component,
    tag: &str,
) -> Deployment {
    let environment_variables =
        Vec::from_iter(component.environment_variables().iter())
            .iter()
            .map(|entry| EnvVar {
                name: entry.0.clone(),
                value_from: Some(EnvVarSource {
                    secret_key_ref: Some(SecretKeySelector {
                        name: Some(component.resource_name().to_owned()),
                        key: entry.0.to_owned(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .collect();

    let labels = default_labels(env, component);

    Deployment {
        metadata: ObjectMeta {
            name: Some(component.resource_name().to_owned()),
            namespace: Some(PROJECT.to_owned()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            selector: LabelSelector {
                match_labels: Some(labels.clone()),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: component.resource_name().to_owned(),
                        image: Some(component.image(image_registry, tag)),
                        env: Some(environment_variables),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        status: None,
    }
}

pub fn expected_secret(env: Environment, component: &Component) -> Secret {
    let labels = default_labels(env, component);

    Secret {
        metadata: ObjectMeta {
            name: Some(component.resource_name().to_owned()),
            namespace: Some(PROJECT.to_owned()),
            labels: Some(labels),
            ..Default::default()
        },
        data: Some(
            component
                .environment_variables()
                .iter()
                .map(|e| (e.0.to_owned(), ByteString(e.1.clone().into_bytes())))
                .collect(),
        ),
        ..Default::default()
    }
}

pub fn default_labels(
    env: Environment,
    component: &Component,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("app.kubernetes.io/name".to_owned(), PROJECT.to_owned()),
        (
            "app.kubernetes.io/component".to_owned(),
            component.resource_name().to_owned(),
        ),
        ("environment".to_owned(), env.to_string()),
        (
            "app.kubernetes.io/version".to_owned(),
            built_info::GIT_VERSION.unwrap_or("unknown").to_owned(),
        ),
    ])
}

pub fn default_namespace_labels(env: Environment) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("app.kubernetes.io/name".to_owned(), PROJECT.to_owned()),
        ("environment".to_owned(), env.to_string()),
        (
            "app.kubernetes.io/version".to_owned(),
            built_info::GIT_VERSION.unwrap_or("unknown").to_owned(),
        ),
    ])
}
