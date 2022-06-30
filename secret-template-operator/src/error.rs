#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    Kube(#[from] kube::Error),
    #[error("Base64 Decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("RandStr error: {0}")]
    RandStr(#[from] randstr::Error),
    #[error("secret key selector has no name")]
    SecretKeySelectorHasNoName,
    #[error("secret has no data")]
    SecretHasNoData,
    #[error("key not found")]
    SecretKeyNotFound,
}
