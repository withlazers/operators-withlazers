macro_rules! operator {
    () => {
        concat!("dev.withlazers.operator.", env!("CARGO_PKG_NAME"))
    };
}

macro_rules! k8s_identifier {
    ($name:literal) => {
        concat!(operator!(), "/", $name)
    };
    () => {
        k8s_identifier!("")
    };
}

pub const ANNOTATION_PREFIX: &str = k8s_identifier!("");
pub const ANNOTATION_ENABLED: &str = k8s_identifier!("enabled");
pub const ANNOTATION_CLONED_FROM: &str =
    k8s_identifier!("cloned_from_namespace");
pub const ANNOTATION_NAMESPACES: &str = k8s_identifier!("namespace");
pub const ANNOTATION_NAMESPACES_DENY: &str = k8s_identifier!("namespace_deny");
