//! The arguments of commands that plugins define.

use rosc::OscType;

/// One argument that scsynth hands to a plugin as it is: an argument of
/// `/cmd`, of `/u_cmd`, or of a plugin's fill for `/b_gen`. The variants are
/// the types that scsynth's argument reader takes without losing its place.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Argument {
    /// A 32-bit integer, `i`.
    Int(i32),
    /// A 32-bit float, `f`.
    Float(f32),
    /// A 64-bit float, `d`.
    Double(f64),
    /// A string, `s`.
    String(String),
    /// A blob, `b`.
    Blob(Vec<u8>),
}

impl Argument {
    pub(crate) fn to_osc(&self) -> OscType {
        match self {
            Self::Int(value) => OscType::Int(*value),
            Self::Float(value) => OscType::Float(*value),
            Self::Double(value) => OscType::Double(*value),
            Self::String(value) => OscType::String(value.clone()),
            Self::Blob(value) => OscType::Blob(value.clone()),
        }
    }

    pub(crate) fn from_osc(value: &OscType) -> Option<Self> {
        match value {
            OscType::Int(value) => Some(Self::Int(*value)),
            OscType::Float(value) => Some(Self::Float(*value)),
            OscType::Double(value) => Some(Self::Double(*value)),
            OscType::String(value) => Some(Self::String(value.clone())),
            OscType::Blob(value) => Some(Self::Blob(value.clone())),
            _ => None,
        }
    }
}
