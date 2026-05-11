use std::{fmt, str::FromStr};

use snafu::{Snafu, ensure};

use crate::key::{self, KeyAlgorithm};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyId(String);

pub type Result<T, E = KeyIdError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum KeyIdError {
    #[snafu(display("expected key id in the form 'algorithm:encoded'"))]
    MissingColon,

    #[snafu(display("key id version part is empty"))]
    MissingVersion,

    #[snafu(display("unknown key algorithm '{algorithm}'"))]
    InvalidAlgorithm { algorithm: String },
}

impl KeyId {
    pub fn new(encoded: String) -> Result<Self> {
        let Some((a, b)) = encoded.split_once(":") else {
            return MissingColonSnafu.fail();
        };
        ensure!(
            key::KeyAlgorithm::from_str(a).is_ok(),
            InvalidAlgorithmSnafu { algorithm: a }
        );
        ensure!(!b.is_empty(), MissingVersionSnafu);
        Ok(Self(encoded))
    }
    pub fn components(&self) -> (KeyAlgorithm, &str) {
        let Some((a, b)) = self.0.split_once(":") else {
            unreachable!("Should be valid once constructed")
        };
        (
            key::KeyAlgorithm::from_str(a).expect("should be valid once constructed"),
            b,
        )
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for KeyId {
    type Err = KeyIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl TryFrom<&str> for KeyId {
    type Error = KeyIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for KeyId {
    type Error = KeyIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<KeyId> for String {
    fn from(value: KeyId) -> Self {
        value.to_string()
    }
}
