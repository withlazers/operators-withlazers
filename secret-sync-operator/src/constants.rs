macro_rules! operator_prefix {
    ($name:literal) => {
        concat!("dev.withlazers.", env!("CARGO_PKG_NAME"), "/", $name)
    };
}

pub const ANNOTATION_PREFIX: &str = operator_prefix!("");
pub const ANNOTATION_ENABLED: &str = operator_prefix!("enabled");
pub const ANNOTATION_CLONED_FROM: &str =
    operator_prefix!("cloned_from_namespace");
pub const ANNOTATION_NAMESPACES: &str = operator_prefix!("namespace");
pub const ANNOTATION_NAMESPACES_DENY: &str = operator_prefix!("namespace_deny");
