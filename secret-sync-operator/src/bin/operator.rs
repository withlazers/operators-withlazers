use futures::StreamExt;
use k8s_openapi::{
    api::core::v1::{Namespace, Secret},
    Metadata,
};
use kube::{
    api::{ListParams, Patch, PatchParams},
    core::ObjectMeta,
    runtime::{controller::Action, Controller},
    Api, Client, ResourceExt,
};
use log::{debug, info, warn};
use secret_sync_operator::{constants::*, globlist::GlobList, result::Result};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

struct Context(Client);

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let client = Client::try_default().await?;
    let secrets = Api::<Secret>::all(client.clone());
    let namespaces = Api::<Namespace>::all(client.clone());

    let context = Arc::new(Context(client));
    futures::join!(
        Controller::new(secrets, ListParams::default())
            .run(reconcile_secret, default_error_policy, context.clone())
            .for_each(|_| futures::future::ready(())),
        Controller::new(namespaces, ListParams::default())
            .run(reconcile_namespace, default_error_policy, context.clone())
            .for_each(|_| futures::future::ready(()))
    );

    Ok(())
}

pub fn default_error_policy<E: std::fmt::Debug, D>(
    _error: &E,
    _ctx: Arc<D>,
) -> Action {
    //Err::<(), _>(error).unwrap();
    Action::requeue(Duration::from_secs(5))
}

fn can_handle(annotations: &Option<BTreeMap<String, String>>) -> bool {
    let annotations = if let Some(annotations) = annotations {
        annotations
    } else {
        return false;
    };

    if !annotations.contains_key(ANNOTATION_ENABLED) {
        return false;
    } else if annotations.contains_key(ANNOTATION_CLONED_FROM) {
        return false;
    } else {
        return true;
    }
}

fn get_globbers(
    annotations: &Option<BTreeMap<String, String>>,
) -> Result<(Option<GlobList>, Option<GlobList>)> {
    match annotations
        .as_ref()
        .map(|annotations| {
            (
                annotations.get(ANNOTATION_NAMESPACES),
                annotations.get(ANNOTATION_NAMESPACES_DENY),
            )
        })
        .map(|(allow, deny)| {
            (
                allow.map(|s| GlobList::new(s)),
                deny.map(|s| GlobList::new(s)),
            )
        })
        .unwrap_or((None, None))
    {
        (Some(Err(err)), _) | (_, Some(Err(err))) => Err(err)?,
        (allow, deny) => {
            Ok((allow.and_then(|a| a.ok()), deny.and_then(|d| d.ok())))
        }
    }
}

async fn sync_secret(
    secrets: &[Secret],
    namespaces: &[Namespace],
    ctx: Arc<Context>,
) -> Result<Action> {
    let client = &ctx.0;

    for secret in secrets {
        if !can_handle(&secret.metadata().annotations) {
            return Ok(Action::await_change());
        }
        let secret_name = secret.name();
        info!("reconcile request secret: {}", secret.name());

        let (allow, deny) = get_globbers(&secret.metadata().annotations)?;

        let allow = if let Some(allow) = allow {
            allow
        } else {
            warn!("secret {} has no allow glob list", secret_name);
            continue;
        };

        for namespace in namespaces {
            let namespace_name = namespace.name();

            if !can_handle(&namespace.metadata().annotations) {
                debug!(
                    "{}: namespace {} is not enabled",
                    secret_name, namespace_name
                );
                continue;
            }

            if Some(&namespace_name) == secret.namespace().as_ref() {
                debug!(
                    "{}: namespace {} is the same as secret",
                    secret_name, namespace_name
                );
                continue;
            }

            info!(
                "reconcile request secret: {} for namespace: {}",
                secret.name(),
                namespace_name
            );

            if !allow.is_match(&namespace_name) {
                debug!(
                    "{}: namespace {} is not a covered by allow globs",
                    secret_name, namespace_name
                );
            } else {
                continue;
            }

            if let Some(ref deny_list) = deny {
                if deny_list.is_match(&namespace_name) {
                    debug!(
                        "{}: namespace {} is covered by deny globs",
                        secret_name, namespace_name
                    );
                    continue;
                }
            }

            let mut annotations = secret
                .metadata()
                .annotations
                .clone()
                .unwrap_or_default()
                .into_iter()
                .filter(|(k, _)| !k.starts_with(ANNOTATION_PREFIX))
                .collect::<BTreeMap<_, _>>();
            annotations.insert(
                ANNOTATION_CLONED_FROM.to_string(),
                secret.namespace().unwrap(),
            );
            let secret_api =
                Api::<Secret>::namespaced(client.clone(), &namespace_name);
            let new_secret = Secret {
                metadata: ObjectMeta {
                    name: secret.metadata().name.clone(),
                    labels: secret.metadata().labels.clone(),
                    annotations: Some(annotations),
                    ..Default::default()
                },
                ..secret.clone()
            };
            debug!(
                "creating secret: {} in {}",
                new_secret.name(),
                namespace_name
            );
            secret_api
                .patch(
                    &new_secret.name(),
                    &PatchParams::apply("secret"),
                    &Patch::Apply(&new_secret),
                )
                .await?;
        }
    }

    info!("reconcile request done");
    Ok(Action::await_change())
}

async fn reconcile_secret(
    obj: Arc<Secret>,
    ctx: Arc<Context>,
) -> Result<Action> {
    let client = &ctx.0;
    let namespaces = Api::<Namespace>::all(client.clone())
        .list(&ListParams::default())
        .await?
        .items;

    sync_secret(&[obj.as_ref().clone()], &namespaces, ctx).await
}

async fn reconcile_namespace(
    obj: Arc<Namespace>,
    ctx: Arc<Context>,
) -> Result<Action> {
    let client = &ctx.0;
    let secrets = Api::<Secret>::all(client.clone())
        .list(&ListParams::default())
        .await?
        .items;

    sync_secret(&secrets, &[obj.as_ref().clone()], ctx).await
}
