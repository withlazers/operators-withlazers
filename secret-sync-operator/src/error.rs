use crate::globlist;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    Kube(#[from] kube::Error),
    #[error("Glob Error: {0}")]
    Glob(#[from] globlist::Error),
}
