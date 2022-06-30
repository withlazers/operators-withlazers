use json_patch::{PatchOperation, RemoveOperation, TestOperation};
use k8s_openapi::api::core::v1::{Secret, SecretKeySelector};
use k8s_openapi::ByteString;
use kube::api::{ListParams, Patch, PatchParams, PostParams};
use kube::core::object::HasSpec;
use kube::core::{ErrorResponse, ObjectMeta};
use kube::runtime::controller::Action;
use kube::runtime::Controller;
use kube::{Api, Client, Resource, ResourceExt};
use secret_template_operator::crd::GenerateDefinition;
use secret_template_operator::{
    crd::DataTemplates, crd::SecretTemplate, error::Error, result::Result,
};
use tokio::join;

use std::collections::BTreeMap;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;

struct Context(Client);

static FINALIZER_NAME: &str =
    "secret-template-operator.withlazers.dev/finalizer";

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let client = Client::try_default().await?;
    let secret_template = Api::<SecretTemplate>::all(client.clone());
    let secret = Api::<Secret>::all(client.clone());

    let context = Arc::new(Context(client));
    join!(
        Controller::new(secret_template, ListParams::default())
            .run(reconcile_secret_template, error_policy, context.clone())
            .for_each(|_| futures::future::ready(())),
        Controller::new(secret, ListParams::default())
            .run(reconcile_secret, error_policy, context.clone())
            .for_each(|_| futures::future::ready(()))
    );

    Ok(())
}

pub fn error_policy<E: Display, D>(error: &E, _ctx: Arc<D>) -> Action {
    println!("{}", error);
    Action::requeue(Duration::from_secs(5))
}

async fn reconcile_secret(
    obj: Arc<Secret>,
    ctx: Arc<Context>,
) -> Result<Action> {
    let client = &ctx.0;

    let ns = obj.namespace().clone().unwrap();
    if obj.meta().deletion_timestamp.is_none() {
        return Ok(Action::await_change());
    }
    // We must run as the last finalizer, so we can recreate the secret.
    if obj.meta().finalizers != Some(vec![FINALIZER_NAME.to_string()]) {
        return Ok(Action::await_change());
    }
    let mut obj = obj.as_ref().clone();
    obj.meta_mut().finalizers = vec![].into();
    obj.meta_mut().managed_fields = None;
    let api = Api::<Secret>::namespaced(client.clone(), &ns);

    let finalizer_path = "/metadata/finalizers/0";
    api.patch(
        &obj.name(),
        &PatchParams::default(),
        &Patch::<SecretTemplate>::Json(json_patch::Patch(vec![
            PatchOperation::Test(TestOperation {
                path: finalizer_path.to_string(),
                value: FINALIZER_NAME.into(),
            }),
            PatchOperation::Remove(RemoveOperation {
                path: finalizer_path.to_string(),
            }),
        ])),
    )
    .await?;

    let api = Api::<SecretTemplate>::namespaced(client.clone(), &ns);
    let owner_refs = obj.meta().owner_references.as_ref().unwrap();
    let owner_ref = owner_refs
        .iter()
        .find(|x| {
            x.kind == SecretTemplate::kind(&())
                && x.api_version == format!("{}/v1", SecretTemplate::group(&()))
        })
        .unwrap();
    let secret_template = api.get(&owner_ref.name).await?;

    reconcile_secret_template(Arc::new(secret_template), ctx).await?;

    Ok(Action::await_change())
}

async fn reconcile_secret_template(
    obj: Arc<SecretTemplate>,
    ctx: Arc<Context>,
) -> Result<Action> {
    let client = &ctx.0;

    let ns = obj.namespace().clone().unwrap();
    let mut data = BTreeMap::new();
    for (key, value) in obj.spec().data.iter() {
        let value = match value {
            DataTemplates::SecretRef(r) => {
                load_secret(&client, &r, &ns).await?
            }
            DataTemplates::Base64(v) => base64::decode(v)?.to_vec(),
            DataTemplates::Plain(v) => v.clone().into_bytes(),
            DataTemplates::Generate(v) => generate_secret(&v)?,
        };
        data.insert(key.clone(), ByteString(value));
    }

    println!(
        "Reconciling secret template: {} in {}",
        obj.name(),
        obj.namespace().unwrap()
    );

    let oref = obj.controller_owner_ref(&()).unwrap();
    let secret = Secret {
        metadata: ObjectMeta {
            namespace: obj.namespace().clone(),
            name: obj.name().clone().into(),
            labels: obj.spec().labels.clone(),
            annotations: obj.spec().annotations.clone(),
            owner_references: vec![oref].into(),
            finalizers: vec![FINALIZER_NAME.to_string()].into(),
            ..Default::default()
        },

        data: data.into(),
        type_: obj.spec().type_.clone(),
        immutable: true.into(),
        ..Default::default()
    };

    let api = Api::<Secret>::namespaced(client.clone(), &ns);

    match api.create(&PostParams::default(), &secret).await {
        Ok(_) => {
            println!("Created secret {}", secret.name());
            Ok(Action::await_change())
        }
        Err(kube::Error::Api(ErrorResponse { code: 409, .. })) => {
            println!("Secret already exists");
            Ok(Action::await_change())
        }
        Err(e) => Err(e.into()),
    }
}

fn generate_secret(v: &GenerateDefinition) -> Result<Vec<u8>> {
    let mut config = randstr::randstr();
    config.len(v.length);
    if let Some(alphabet) = &v.custom_alphabet {
        if v.must_custom_alphabet.unwrap_or(false) {
            config.must_custom(&alphabet);
        } else {
            config.custom(&alphabet);
        }
    }

    if v.must_letters.unwrap_or(false) {
        config.must_letter();
    } else if v.letters.unwrap_or(false) {
        config.letter();
    }

    if v.must_digits.unwrap_or(false) {
        config.must_digit();
    } else if v.digits.unwrap_or(false) {
        config.digit();
    }

    if v.must_symbols.unwrap_or(false) {
        config.must_symbol();
    } else if v.symbols.unwrap_or(false) {
        config.symbol();
    }

    if v.must_uppercase.unwrap_or(false) {
        config.must_upper();
    } else if v.uppercase.unwrap_or(false) {
        config.upper();
    }

    if v.must_lowercase.unwrap_or(false) {
        config.must_lower();
    } else if v.lowercase.unwrap_or(false) {
        config.lower();
    }

    let mut generator = config.try_build()?;

    Ok(generator.generate().into_bytes())
}

async fn load_secret(
    client: &Client,
    r: &SecretKeySelector,
    ns: &str,
) -> Result<Vec<u8>> {
    let api: Api<Secret> = Api::namespaced(client.clone(), ns);
    let secret = api
        .get(&r.name.clone().ok_or(Error::SecretKeySelectorHasNoName)?)
        .await?;
    let datas = secret.data.ok_or(Error::SecretHasNoData)?;
    let data = datas.get(&r.key).ok_or(Error::SecretKeyNotFound)?;

    Ok(data.0.clone())
}
