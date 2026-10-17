//! The values that many messages share: nodes, add actions, controls and
//! their values.

use std::fmt;
use std::str::FromStr;

use rosc::{OscArray, OscType};

use crate::codec::{Reader, Writer, coded, tag};
use crate::error::DecodeError;

/// A node of the server's tree: a synth or a group. Node 0 is the root
/// group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct NodeId(pub i32);

impl NodeId {
    /// The root group.
    pub const ROOT: Self = Self(0);

    /// In a command, -1 names the node that the server created last. It is
    /// how a client reaches a node whose ID the server picked.
    pub const RECENT: Self = Self(-1);

    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.int().map(Self)
    }

    pub(crate) fn write(self, writer: &mut Writer) {
        writer.int(self.0);
    }
}

/// The ID of a node to create: one that the client picks, 0 or more, or
/// one that the server picks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "i32", into = "i32")
)]
pub struct NewNodeId(i32);

impl NewNodeId {
    /// The server picks a negative ID. It sends no notification for such a
    /// node, and only [`NodeId::RECENT`] names it, until the next node is
    /// created.
    pub const AUTO: Self = Self(-1);

    /// `id`, when it is 0 or more. scsynth refuses the other negative IDs.
    pub fn new(id: i32) -> Option<Self> {
        (id >= 0).then_some(Self(id))
    }

    /// The ID on the wire: -1 for [`NewNodeId::AUTO`].
    pub fn get(self) -> i32 {
        self.0
    }

    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let index = reader.index();
        let id = reader.int()?;
        Self::try_from(id).map_err(|_| {
            reader.invalid(
                index,
                format!("{id} is no new node ID: -1 or an ID of 0 or more"),
            )
        })
    }

    pub(crate) fn write(self, writer: &mut Writer) {
        writer.int(self.0);
    }
}

impl TryFrom<i32> for NewNodeId {
    type Error = i32;

    /// -1, for [`NewNodeId::AUTO`], or an ID of 0 or more.
    fn try_from(id: i32) -> Result<Self, i32> {
        if id == -1 {
            Ok(Self::AUTO)
        } else {
            Self::new(id).ok_or(id)
        }
    }
}

impl From<NewNodeId> for i32 {
    fn from(id: NewNodeId) -> Self {
        id.0
    }
}

coded! {
    /// Where a new node goes, relative to its target.
    AddAction {
        /// At the head of the target, a group.
        Head = 0,
        /// At the tail of the target, a group.
        Tail = 1,
        /// Just before the target.
        Before = 2,
        /// Just after the target.
        After = 3,
        /// In the place of the target, which the server frees. scsynth
        /// sends no `/n_end` for the target.
        Replace = 4,
    }
}

impl AddAction {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.coded("an add action, 0 to 4", Self::from_code)
    }
}

coded! {
    /// Where `/n_order` places its nodes, relative to its target.
    OrderAction {
        /// At the head of the target, a group.
        Head = 0,
        /// At the tail of the target, a group.
        Tail = 1,
        /// Just before the target.
        Before = 2,
        /// Just after the target.
        After = 3,
    }
}

impl OrderAction {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        reader.coded("an add action of /n_order, 0 to 3", Self::from_code)
    }
}

/// A control of a synth: by its index, or by its name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum Control {
    /// The index of the control.
    Index(i32),
    /// The name of the control.
    Name(String),
}

impl Control {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        const EXPECTED: &str = "a control index or name";
        match reader.any(EXPECTED)? {
            OscType::Int(index) => Ok(Self::Index(*index)),
            OscType::String(name) => Ok(Self::Name(name.clone())),
            other => Err(reader.wrong(other, EXPECTED)),
        }
    }

    pub(crate) fn write(&self, writer: &mut Writer) {
        match self {
            Self::Index(index) => writer.int(*index),
            Self::Name(name) => writer.string(name),
        }
    }
}

impl From<i32> for Control {
    fn from(index: i32) -> Self {
        Self::Index(index)
    }
}

impl From<&str> for Control {
    fn from(name: &str) -> Self {
        Self::Name(name.to_owned())
    }
}

impl From<String> for Control {
    fn from(name: String) -> Self {
        Self::Name(name)
    }
}

/// An int or a float, where the reference takes either. scsynth reads both
/// as a float.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum Number {
    /// An int.
    Int(i32),
    /// A float.
    Float(f32),
}

impl Number {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        const EXPECTED: &str = "an int or a float";
        match reader.any(EXPECTED)? {
            OscType::Int(value) => Ok(Self::Int(*value)),
            OscType::Float(value) => Ok(Self::Float(*value)),
            other => Err(reader.wrong(other, EXPECTED)),
        }
    }

    pub(crate) fn write(self, writer: &mut Writer) {
        match self {
            Self::Int(value) => writer.int(value),
            Self::Float(value) => writer.float(value),
        }
    }

    /// The same value with the same tag: a NaN equals itself here.
    pub(crate) fn same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}

impl From<i32> for Number {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<f32> for Number {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

/// A bus for a control to read, in the place of a value. It travels as the
/// string `c<index>` for a control bus, or `a<index>` for an audio bus.
///
/// scsynth reads the index as an `i32`: it takes an index above `i32::MAX`
/// as a negative bus, which undoes the mapping. Decoding takes only the
/// forms of the reference, although scsynth reads `cx` as bus 0 and `c`
/// alone as no mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "String", into = "String")
)]
pub enum BusMapping {
    /// A control bus.
    Control(u32),
    /// An audio bus.
    Audio(u32),
}

impl BusMapping {
    /// The mapping in `text`, the argument at `index`.
    pub(crate) fn parse_at(
        reader: &Reader<'_>,
        index: usize,
        text: &str,
    ) -> Result<Self, DecodeError> {
        text.parse()
            .map_err(|error: NotABusMapping| reader.invalid(index, error.to_string()))
    }
}

impl fmt::Display for BusMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Control(index) => write!(f, "c{index}"),
            Self::Audio(index) => write!(f, "a{index}"),
        }
    }
}

impl FromStr for BusMapping {
    type Err = NotABusMapping;

    /// `c` or `a`, then the index in decimal digits.
    fn from_str(text: &str) -> Result<Self, NotABusMapping> {
        let refuse = || NotABusMapping(text.to_owned());
        let (rate, digits) = text.split_at_checked(1).ok_or_else(refuse)?;
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(refuse());
        }
        let index = digits.parse().map_err(|_| refuse())?;
        match rate {
            "c" => Ok(Self::Control(index)),
            "a" => Ok(Self::Audio(index)),
            _ => Err(refuse()),
        }
    }
}

impl TryFrom<String> for BusMapping {
    type Error = NotABusMapping;

    fn try_from(text: String) -> Result<Self, NotABusMapping> {
        text.parse()
    }
}

impl From<BusMapping> for String {
    fn from(mapping: BusMapping) -> Self {
        mapping.to_string()
    }
}

/// A string that is no [`BusMapping`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotABusMapping(pub String);

impl fmt::Display for NotABusMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} is no bus mapping: 'c' or 'a', then the index of the bus in decimal digits",
            self.0
        )
    }
}

impl std::error::Error for NotABusMapping {}

/// The value of a control in `/s_new` and `/n_set`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum ControlValue {
    /// An int, which scsynth sets as a float.
    Int(i32),
    /// A float.
    Float(f32),
    /// A bus for the control to read.
    Bus(BusMapping),
    /// Values for the control and for the controls after it, in order, as
    /// `/n_setn` sets them. The array has one level: scsynth reads a nested
    /// array differently in `/s_new` and in `/n_set`.
    Array(Vec<ArrayElement>),
}

/// One value in a [`ControlValue::Array`].
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum ArrayElement {
    /// An int, which scsynth sets as a float.
    Int(i32),
    /// A float.
    Float(f32),
    /// A bus for the control to read.
    Bus(BusMapping),
}

impl ControlValue {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        const EXPECTED: &str = "a control value: an int, a float, a bus mapping or an array";
        let index = reader.index();
        match reader.any(EXPECTED)? {
            OscType::Int(value) => Ok(Self::Int(*value)),
            OscType::Float(value) => Ok(Self::Float(*value)),
            OscType::String(text) => BusMapping::parse_at(reader, index, text).map(Self::Bus),
            OscType::Array(array) => array
                .content
                .iter()
                .map(|element| ArrayElement::read(reader, index, element))
                .collect::<Result<_, _>>()
                .map(Self::Array),
            other => Err(reader.wrong(other, EXPECTED)),
        }
    }

    pub(crate) fn write(&self, writer: &mut Writer) {
        writer.push(match self {
            Self::Int(value) => OscType::Int(*value),
            Self::Float(value) => OscType::Float(*value),
            Self::Bus(mapping) => OscType::String(mapping.to_string()),
            Self::Array(elements) => OscType::Array(OscArray {
                content: elements.iter().copied().map(ArrayElement::to_osc).collect(),
            }),
        });
    }
}

impl ArrayElement {
    /// The element `value` of the array at `index`.
    fn read(reader: &Reader<'_>, index: usize, value: &OscType) -> Result<Self, DecodeError> {
        match value {
            OscType::Int(value) => Ok(Self::Int(*value)),
            OscType::Float(value) => Ok(Self::Float(*value)),
            OscType::String(text) => BusMapping::parse_at(reader, index, text).map(Self::Bus),
            OscType::Array(_) => Err(reader.invalid(
                index,
                "a nested array, which scsynth reads differently in /s_new and /n_set",
            )),
            other => Err(reader.invalid(
                index,
                format!(
                    "an element of type '{}' in an array, where scsynth loops forever",
                    tag(other)
                ),
            )),
        }
    }

    fn to_osc(self) -> OscType {
        match self {
            Self::Int(value) => OscType::Int(value),
            Self::Float(value) => OscType::Float(value),
            Self::Bus(mapping) => OscType::String(mapping.to_string()),
        }
    }
}

impl From<i32> for ControlValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<f32> for ControlValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<BusMapping> for ControlValue {
    fn from(mapping: BusMapping) -> Self {
        Self::Bus(mapping)
    }
}

/// A control and its value, in `/s_new` and `/n_set`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlSetting {
    /// The control.
    pub control: Control,
    /// Its value.
    pub value: ControlValue,
}

impl ControlSetting {
    /// `control` set to `value`.
    pub fn new(control: impl Into<Control>, value: impl Into<ControlValue>) -> Self {
        Self {
            control: control.into(),
            value: value.into(),
        }
    }

    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            control: Control::read(reader)?,
            value: ControlValue::read(reader)?,
        })
    }

    pub(crate) fn write(&self, writer: &mut Writer) {
        self.control.write(writer);
        self.value.write(writer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bus_mapping_is_a_rate_and_decimal_digits() {
        for (text, mapping) in [
            ("c0", BusMapping::Control(0)),
            ("c12", BusMapping::Control(12)),
            ("a3", BusMapping::Audio(3)),
            ("c007", BusMapping::Control(7)),
            ("a4294967295", BusMapping::Audio(u32::MAX)),
        ] {
            assert_eq!(text.parse(), Ok(mapping), "{text}");
        }
        for text in [
            "",
            "c",
            "a",
            "cx",
            "c12x",
            "c-1",
            "c+1",
            "C1",
            "x1",
            "c 1",
            "c4294967296",
            "é1",
        ] {
            assert_eq!(
                text.parse::<BusMapping>(),
                Err(NotABusMapping(text.to_owned())),
                "{text}"
            );
        }
        assert_eq!(BusMapping::Audio(3).to_string(), "a3");
    }

    #[test]
    fn a_new_node_id_is_minus_1_or_0_or_more() {
        assert_eq!(NewNodeId::try_from(-1), Ok(NewNodeId::AUTO));
        assert_eq!(NewNodeId::try_from(0).map(NewNodeId::get), Ok(0));
        assert_eq!(NewNodeId::try_from(-2), Err(-2));
        assert_eq!(NewNodeId::new(-1), None);
    }
}
