use ed25519_compact::PublicKey as Ed25519PublicKey;
use httpsig::prelude::{
    AlgorithmName as HttpSigAlgorithmName, HttpSigError, PublicKey as HttpSigPublicKey,
};
use strum::IntoDiscriminant;
#[derive(Clone, Debug, strum::EnumDiscriminants)]
#[strum_discriminants(
    name(KeyAlgorithm),
    derive(
        Hash,
        strum::Display,
        strum::AsRefStr,
        strum::EnumString,
        strum::VariantArray
    ),
    strum(serialize_all = "snake_case")
)]
#[strum(serialize_all = "snake_case")]
pub enum PublicKey {
    Ed25519(Ed25519PublicKey),
}
impl PublicKey {
    pub fn key_id(&self) -> String {
        use base64::Engine;
        use sha2::{Digest, Sha256};
        let bytes = match self {
            Self::Ed25519(vk) => vk.as_ref().to_vec(),
        };
        let mut hasher = <Sha256 as Digest>::new();
        hasher.update(&bytes);
        let hash = hasher.finalize();

        format!(
            "{}:{}",
            self.discriminant(),
            &base64::engine::general_purpose::STANDARD_NO_PAD.encode(hash)
        )
    }
}

impl From<PublicKey> for HttpSigPublicKey {
    fn from(value: PublicKey) -> Self {
        match value {
            PublicKey::Ed25519(k) => HttpSigPublicKey::Ed25519(k),
        }
    }
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
