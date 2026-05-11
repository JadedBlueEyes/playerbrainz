use httpsig::prelude::{AlgorithmName as HttpSigAlgorithmName, HttpSigError};

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    strum::Display,
    strum::AsRefStr,
    strum::EnumString,
    strum::VariantArray,
)]
#[strum(serialize_all = "snake_case")]
pub enum KeyAlgorithm {
    Ed25519,
}

impl From<KeyAlgorithm> for HttpSigAlgorithmName {
    fn from(value: KeyAlgorithm) -> Self {
        match value {
            KeyAlgorithm::Ed25519 => HttpSigAlgorithmName::Ed25519,
        }
    }
}

impl TryFrom<HttpSigAlgorithmName> for KeyAlgorithm {
    type Error = HttpSigError;

    fn try_from(value: HttpSigAlgorithmName) -> Result<Self, Self::Error> {
        match value {
            HttpSigAlgorithmName::Ed25519 => Ok(KeyAlgorithm::Ed25519),
            a => Err(HttpSigError::InvalidAlgorithmName(format!(
                "Algorithm {a:?} is not implemented"
            ))),
        }
    }
}
