use std::str::FromStr as _;

use nanoid::nanoid;
use sea_orm::DeriveValueType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
#[sea_orm(value_type = "String")]
pub struct NanoId(pub(crate) String);

const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub const NANO_ID_LEN: usize = 6;

impl Default for NanoId {
    fn default() -> Self {
        Self(nanoid!(NANO_ID_LEN, &ALPHABET))
    }
}

#[derive(Debug)]
pub struct NanoIdParseError {
    bad: String,
}

impl std::error::Error for NanoIdParseError {}

impl std::fmt::Display for NanoIdParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} is not a valid NanoId!", self.bad)
    }
}

impl std::fmt::Display for NanoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for NanoId {
    type Err = NanoIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

impl From<&str> for NanoId {
    fn from(value: &str) -> Self {
        NanoId::from_str(value).unwrap()
    }
}
