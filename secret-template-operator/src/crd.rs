use std::collections::{BTreeMap, HashMap};

use k8s_openapi::api::core::v1::SecretKeySelector;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[kube(
    group = "crd.withlazers.dev",
    version = "v1",
    kind = "SecretTemplate",
    namespaced
)]
#[serde(rename_all = "snake_case")]
pub struct SecretTemplateSpec {
    pub labels: Option<BTreeMap<String, String>>,
    pub annotations: Option<BTreeMap<String, String>>,
    pub data: HashMap<String, DataTemplates>,
    pub type_: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataTemplates {
    SecretRef(SecretKeySelector),
    Base64(String),
    Plain(String),
    Generate(GenerateDefinition),
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GenerateDefinition {
    pub length: usize,
    #[serde(default)]
    pub custom_alphabet: Option<String>,
    #[serde(default)]
    pub letters: Option<bool>,
    #[serde(default)]
    pub digits: Option<bool>,
    #[serde(default)]
    pub symbols: Option<bool>,
    #[serde(default)]
    pub uppercase: Option<bool>,
    #[serde(default)]
    pub lowercase: Option<bool>,

    #[serde(default)]
    pub must_letters: Option<bool>,
    #[serde(default)]
    pub must_digits: Option<bool>,
    #[serde(default)]
    pub must_symbols: Option<bool>,
    #[serde(default)]
    pub must_custom_alphabet: Option<bool>,
    #[serde(default)]
    pub must_uppercase: Option<bool>,
    #[serde(default)]
    pub must_lowercase: Option<bool>,
}
