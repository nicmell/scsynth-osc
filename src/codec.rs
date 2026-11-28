//! The strict reading and the canonical writing of arguments, and what the
//! enums of `messages!` share.

use rosc::{OscMessage, OscType};

use crate::error::{DecodeError, DecodeErrorKind};
use crate::packet::{decode_message, encode_message};
use crate::raw::{RawArg, RawMessage, osc_message, raw_arg, up_to_nul};

/// The OSC type tag of an argument.
pub(crate) fn tag(value: &OscType) -> char {
    match value {
        OscType::Int(_) => 'i',
        OscType::Float(_) => 'f',
        OscType::String(_) => 's',
        OscType::Blob(_) => 'b',
        OscType::Time(_) => 't',
        OscType::Long(_) => 'h',
        OscType::Double(_) => 'd',
        OscType::Char(_) => 'c',
        OscType::Color(_) => 'r',
        OscType::Midi(_) => 'm',
        OscType::Bool(true) => 'T',
        OscType::Bool(false) => 'F',
        OscType::Array(_) => '[',
        OscType::Nil => 'N',
        OscType::Inf => 'I',
    }
}

/// A cursor over the arguments of one message, which a [`Payload`] reads.
/// It takes each argument only in the type that the protocol gives it.
pub struct Reader<'a> {
    address: &'a str,
    args: &'a [OscType],
    index: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(address: &'a str, args: &'a [OscType]) -> Self {
        Self {
            address,
            args,
            index: 0,
        }
    }

    pub(crate) fn error(&self, kind: DecodeErrorKind) -> DecodeError {
        DecodeError::new(self.address, kind)
    }

    /// The error of the argument at `index`, whose value the protocol does
    /// not take, for `reason`.
    #[must_use]
    pub fn invalid(&self, index: usize, reason: impl Into<String>) -> DecodeError {
        self.error(DecodeErrorKind::Invalid {
            index,
            reason: reason.into(),
        })
    }

    /// The position of the next argument.
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Whether an argument is left.
    #[must_use]
    pub fn remains(&self) -> bool {
        self.index < self.args.len()
    }

    pub(crate) fn peek(&self) -> Option<&'a OscType> {
        self.args.get(self.index)
    }

    /// The next argument, of any type.
    pub(crate) fn any(&mut self, expected: &'static str) -> Result<&'a OscType, DecodeError> {
        let value = self.args.get(self.index).ok_or_else(|| {
            self.error(DecodeErrorKind::Missing {
                index: self.index,
                expected,
            })
        })?;
        self.index += 1;
        Ok(value)
    }

    /// `value`, the argument just taken, is not `expected`.
    pub(crate) fn wrong_osc(&self, value: &OscType, expected: &'static str) -> DecodeError {
        self.error(DecodeErrorKind::WrongType {
            index: self.index - 1,
            expected,
            found: tag(value),
        })
    }

    /// The next argument, of any type. `expected` says what it must be,
    /// for the error when none is left.
    ///
    /// # Errors
    ///
    /// When no argument is left.
    pub fn arg(&mut self, expected: &'static str) -> Result<RawArg, DecodeError> {
        self.any(expected).map(raw_arg)
    }

    /// The error of `value`, the argument that [`Self::arg`] just took,
    /// which is not `expected`.
    #[must_use]
    pub fn wrong(&self, value: &RawArg, expected: &'static str) -> DecodeError {
        self.error(DecodeErrorKind::WrongType {
            index: self.index - 1,
            expected,
            found: value.tag(),
        })
    }

    /// The next argument, an int.
    ///
    /// # Errors
    ///
    /// When no argument is left, or the next one is not an int.
    pub fn int(&mut self) -> Result<i32, DecodeError> {
        match self.any("an int")? {
            OscType::Int(value) => Ok(*value),
            other => Err(self.wrong_osc(other, "an int")),
        }
    }

    /// The next argument, a float.
    ///
    /// # Errors
    ///
    /// When no argument is left, or the next one is not a float.
    pub fn float(&mut self) -> Result<f32, DecodeError> {
        match self.any("a float")? {
            OscType::Float(value) => Ok(*value),
            other => Err(self.wrong_osc(other, "a float")),
        }
    }

    /// The next argument, a double.
    ///
    /// # Errors
    ///
    /// When no argument is left, or the next one is not a double.
    pub fn double(&mut self) -> Result<f64, DecodeError> {
        match self.any("a double")? {
            OscType::Double(value) => Ok(*value),
            other => Err(self.wrong_osc(other, "a double")),
        }
    }

    /// The next argument, a string.
    ///
    /// # Errors
    ///
    /// When no argument is left, or the next one is not a string.
    pub fn string(&mut self) -> Result<String, DecodeError> {
        match self.any("a string")? {
            OscType::String(value) => Ok(value.clone()),
            other => Err(self.wrong_osc(other, "a string")),
        }
    }

    /// The next argument, a blob.
    ///
    /// # Errors
    ///
    /// When no argument is left, or the next one is not a blob.
    pub fn blob(&mut self) -> Result<Vec<u8>, DecodeError> {
        match self.any("a blob")? {
            OscType::Blob(value) => Ok(value.clone()),
            other => Err(self.wrong_osc(other, "a blob")),
        }
    }

    /// The next argument when one is left, which must be an int.
    pub(crate) fn optional_int(&mut self) -> Result<Option<i32>, DecodeError> {
        if self.remains() {
            self.int().map(Some)
        } else {
            Ok(None)
        }
    }

    /// The next argument, an int, or `default` when the message ends here.
    pub(crate) fn int_or(&mut self, default: i32) -> Result<i32, DecodeError> {
        if self.remains() {
            self.int()
        } else {
            Ok(default)
        }
    }

    /// The next argument, a float, or `default` when the message ends here.
    pub(crate) fn float_or(&mut self, default: f32) -> Result<f32, DecodeError> {
        if self.remains() {
            self.float()
        } else {
            Ok(default)
        }
    }

    /// The next argument, a flag, or `default` when the message ends here.
    pub(crate) fn flag_or(&mut self, default: bool) -> Result<bool, DecodeError> {
        if self.remains() {
            self.flag()
        } else {
            Ok(default)
        }
    }

    /// A string that `from_name` knows.
    pub(crate) fn named<T>(
        &mut self,
        expected: &'static str,
        from_name: fn(&str) -> Option<T>,
    ) -> Result<T, DecodeError> {
        let index = self.index;
        let name = self.string()?;
        from_name(&name).ok_or_else(|| self.invalid(index, format!("{name:?} is not {expected}")))
    }

    /// The next argument, an int that stands for a flag: 1 or 0.
    ///
    /// # Errors
    ///
    /// When no argument is left, or the next one is not 1 or 0.
    pub fn flag(&mut self) -> Result<bool, DecodeError> {
        self.coded("1 or 0", |code| match code {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        })
    }

    /// The next argument, an int that counts the items after it: 0 or
    /// more.
    ///
    /// # Errors
    ///
    /// When no argument is left, or the next one is not an int of 0 or
    /// more.
    pub fn count(&mut self) -> Result<usize, DecodeError> {
        let index = self.index;
        let count = self.int()?;
        usize::try_from(count).map_err(|_| self.invalid(index, format!("{count} is no count")))
    }

    /// Every argument left, read by `read` until none is left.
    ///
    /// # Errors
    ///
    /// The first error of `read`.
    pub fn rest<T>(
        &mut self,
        read: impl Fn(&mut Self) -> Result<T, DecodeError>,
    ) -> Result<Vec<T>, DecodeError> {
        let mut values = Vec::new();
        while self.remains() {
            values.push(read(self)?);
        }
        Ok(values)
    }

    /// An int that `from_code` knows.
    pub(crate) fn coded<T>(
        &mut self,
        expected: &'static str,
        from_code: fn(i32) -> Option<T>,
    ) -> Result<T, DecodeError> {
        let index = self.index;
        let code = self.int()?;
        from_code(code).ok_or_else(|| self.invalid(index, format!("{code} is not {expected}")))
    }

    /// No argument may be left.
    pub(crate) fn finish(&self) -> Result<(), DecodeError> {
        if self.remains() {
            Err(self.error(DecodeErrorKind::Trailing { index: self.index }))
        } else {
            Ok(())
        }
    }
}

/// The arguments of one message, as a [`Payload`] writes them.
pub struct Writer {
    args: Vec<RawArg>,
}

impl Writer {
    pub(crate) fn new() -> Self {
        Self { args: Vec::new() }
    }

    /// Writes an int.
    pub fn int(&mut self, value: i32) {
        self.args.push(RawArg::Int(value));
    }

    /// Writes a float.
    pub fn float(&mut self, value: f32) {
        self.args.push(RawArg::Float(value));
    }

    /// Writes a double.
    pub fn double(&mut self, value: f64) {
        self.args.push(RawArg::Double(value));
    }

    /// Writes a string. It travels up to its first NUL byte.
    pub fn string(&mut self, value: &str) {
        self.args.push(RawArg::String(value.to_owned()));
    }

    /// Writes a blob.
    pub fn blob(&mut self, value: Vec<u8>) {
        self.args.push(RawArg::Blob(value));
    }

    /// Writes a flag, as the int 1 or 0.
    pub fn flag(&mut self, value: bool) {
        self.int(i32::from(value));
    }

    /// Writes the length of a list, as the int that counts it.
    ///
    /// # Panics
    ///
    /// When `len` is more than `i32::MAX`, which no OSC packet holds.
    pub fn count(&mut self, len: usize) {
        self.int(i32::try_from(len).expect("an OSC packet holds fewer than 2^31 arguments"));
    }

    /// Writes an argument of any type.
    pub fn push(&mut self, value: RawArg) {
        self.args.push(value);
    }

    pub(crate) fn finish(self) -> Vec<RawArg> {
        self.args
    }
}

/// The arguments of a message: how they are written, and how they are
/// read. A payload reads its arguments from the position of the reader,
/// so payloads compose. The enum of [`messages!`](crate::messages) refuses
/// a message with an argument left after its payload.
pub trait Payload: Sized {
    /// Writes the arguments.
    fn write(&self, writer: &mut Writer);

    /// Reads the arguments at the position of `reader`.
    ///
    /// # Errors
    ///
    /// When an argument is missing, has another type, or has a value that
    /// the protocol does not take.
    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError>;
}

/// What a [`crate::Packet`] holds: a [`crate::Command`], a
/// [`crate::Reply`], a [`crate::RawMessage`], or an enum of
/// [`messages!`](crate::messages). Sealed.
pub trait Message: sealed::Codec {}

pub(crate) mod sealed {
    use rosc::OscMessage;

    use crate::error::DecodeError;
    use crate::raw::{RawMessage, osc_message};

    /// The forms of a [`super::Message`]: raw, and as rosc reads it.
    pub trait Codec: Sized {
        fn to_raw(&self) -> RawMessage;

        fn from_osc(message: &OscMessage) -> Result<Self, DecodeError>;

        fn to_osc(&self) -> OscMessage {
            osc_message(&self.to_raw())
        }

        fn from_raw(raw: &RawMessage) -> Result<Self, DecodeError> {
            Self::from_osc(&osc_message(raw))
        }
    }
}

/// What [`messages!`](crate::messages) implements for each enum that it
/// declares. The output of the macro needs it public, and nothing else
/// does.
#[doc(hidden)]
pub trait Dispatch: Sized {
    /// Every address of the enum.
    const ADDRESSES: &'static [&'static str];

    /// The address of the message.
    fn address(&self) -> &'static str;

    /// Writes the arguments of the message.
    fn write_args(&self, writer: &mut Writer);

    /// Reads the arguments of the message at `address`.
    ///
    /// # Errors
    ///
    /// When the enum has no message at `address`, or its arguments do not
    /// read.
    fn read_args(address: &str, reader: &mut Reader<'_>) -> Result<Self, DecodeError>;
}

impl<T: Dispatch> sealed::Codec for T {
    fn to_raw(&self) -> RawMessage {
        let mut writer = Writer::new();
        self.write_args(&mut writer);
        RawMessage {
            address: self.address().to_owned(),
            args: writer.finish(),
        }
    }

    fn from_osc(message: &OscMessage) -> Result<Self, DecodeError> {
        let mut reader = Reader::new(&message.addr, &message.args);
        let value = T::read_args(&message.addr, &mut reader)?;
        reader.finish()?;
        Ok(value)
    }

    fn from_raw(raw: &RawMessage) -> Result<Self, DecodeError> {
        // The address ends at its first NUL byte, as `osc_message` writes it.
        let address = up_to_nul(&raw.address);
        if !T::ADDRESSES.contains(&address) {
            return Err(unknown_address(address));
        }
        Self::from_osc(&osc_message(raw))
    }
}

impl<T: Dispatch> Message for T {}

/// The bytes of `message`, as one OSC message.
#[doc(hidden)]
#[must_use]
pub fn encode<T: Dispatch>(message: &T) -> Vec<u8> {
    encode_message(sealed::Codec::to_osc(message))
}

/// The message in `bytes`, which must hold exactly one OSC message.
///
/// # Errors
///
/// When the bytes are not one OSC message, or not a message of `T`.
#[doc(hidden)]
pub fn decode<T: Dispatch>(bytes: &[u8]) -> Result<T, DecodeError> {
    <T as sealed::Codec>::from_osc(&decode_message(bytes)?)
}

/// The message as its address and typed arguments.
#[doc(hidden)]
#[must_use]
pub fn to_raw<T: Dispatch>(message: &T) -> RawMessage {
    sealed::Codec::to_raw(message)
}

/// The message in `raw`, decoded as strictly as [`decode`] decodes bytes.
///
/// # Errors
///
/// When `raw` is not a message of `T`.
#[doc(hidden)]
pub fn from_raw<T: Dispatch>(raw: &RawMessage) -> Result<T, DecodeError> {
    <T as sealed::Codec>::from_raw(raw)
}

/// The error of a message at `address`, which the enum does not have.
#[doc(hidden)]
#[must_use]
pub fn unknown_address(address: &str) -> DecodeError {
    DecodeError::new(address, DecodeErrorKind::UnknownAddress)
}

/// Declares an enum of OSC messages: one variant per address, a unit
/// variant when the message has no arguments, a constant per address, and
/// the dispatch by address in both directions. `Command` and `Reply` are
/// enums of it, and an app declares its own: see
/// [Custom messages](crate#custom-messages).
#[macro_export]
macro_rules! messages {
    (serde; $(#[$meta:meta])* $vis:vis enum $($rest:tt)*) => {
        $crate::messages! { @cfg [all()] $(#[$meta])* $vis enum $($rest)* }
    };
    (
        @cfg [$cfg:meta]
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $address:literal as $constant:ident => $variant:ident $(($payload:ty))?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr($cfg, derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr($cfg, serde(tag = "address", content = "args"))]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                #[cfg_attr($cfg, serde(rename = $address))]
                $variant $(($payload))?,
            )*
        }

        impl $name {
            $(
                #[doc = concat!("The address `", $address, "`.")]
                pub const $constant: &'static str = $address;
            )*

            /// Every address of the enum, in the order of its variants.
            pub const ADDRESSES: &'static [&'static str] = &[$($address),*];

            /// The names of the constants of [`Self::ADDRESSES`], in its
            /// order.
            #[doc(hidden)]
            pub const ADDRESS_NAMES: &'static [&'static str] = &[$(stringify!($constant)),*];

            /// The OSC address of the message.
            #[must_use]
            pub fn address(&self) -> &'static str {
                match self {
                    $(Self::$variant { .. } => Self::$constant,)*
                }
            }

            /// The bytes of the message, as one OSC message.
            ///
            /// # Panics
            ///
            /// On a list of more than `i32::MAX` values, which no OSC packet
            /// holds.
            #[must_use]
            pub fn encode(&self) -> ::std::vec::Vec<u8> {
                $crate::__private::encode(self)
            }

            /// The message in `bytes`, which must hold exactly one OSC
            /// message.
            ///
            /// # Errors
            ///
            /// When the bytes are not one OSC message, or not a message of
            /// this type. [`DecodeErrorKind`]($crate::DecodeErrorKind) says
            /// which.
            pub fn decode(bytes: &[u8]) -> ::std::result::Result<Self, $crate::DecodeError> {
                $crate::__private::decode(bytes)
            }

            /// The message as its address and typed arguments. See
            /// [Raw messages]($crate#raw-messages).
            #[must_use]
            pub fn to_raw(&self) -> $crate::RawMessage {
                $crate::__private::to_raw(self)
            }

            /// The message in `raw`, decoded as strictly as
            /// [`Self::decode`] decodes bytes.
            ///
            /// # Errors
            ///
            /// When `raw` is not a message of this type.
            /// [`DecodeErrorKind`]($crate::DecodeErrorKind) says which.
            pub fn from_raw(
                raw: &$crate::RawMessage,
            ) -> ::std::result::Result<Self, $crate::DecodeError> {
                $crate::__private::from_raw(raw)
            }
        }

        impl $crate::__private::Dispatch for $name {
            const ADDRESSES: &'static [&'static str] = <$name>::ADDRESSES;

            fn address(&self) -> &'static str {
                <$name>::address(self)
            }

            // An enum of unit variants alone writes and reads no argument.
            #[allow(unused_variables)]
            fn write_args(&self, writer: &mut $crate::Writer) {
                match self {
                    $(
                        Self::$variant $(($crate::messages!(@bind payload $payload)))? => {
                            $(<$payload as $crate::Payload>::write(payload, writer);)?
                        }
                    )*
                }
            }

            #[allow(unused_variables)]
            fn read_args(
                address: &str,
                reader: &mut $crate::Reader<'_>,
            ) -> ::std::result::Result<Self, $crate::DecodeError> {
                ::std::result::Result::Ok(match address {
                    $(Self::$constant => $crate::messages!(@read reader $variant $($payload)?),)*
                    _ => {
                        return ::std::result::Result::Err($crate::__private::unknown_address(
                            address,
                        ));
                    }
                })
            }
        }
    };
    (@bind $binding:ident $payload:ty) => {
        $binding
    };
    (@read $reader:ident $variant:ident) => {
        Self::$variant
    };
    (@read $reader:ident $variant:ident $payload:ty) => {
        Self::$variant(<$payload as $crate::Payload>::read($reader)?)
    };
    ($(#[$meta:meta])* $vis:vis enum $($rest:tt)*) => {
        $crate::messages! { @cfg [any()] $(#[$meta])* $vis enum $($rest)* }
    };
}

/// Declares an enum of the values that one int argument can code.
macro_rules! coded {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $code:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl $name {
            /// The int that travels on the wire.
            #[must_use]
            pub fn code(self) -> i32 {
                match self {
                    $(Self::$variant => $code,)*
                }
            }

            /// The value of `code`, when the protocol knows it.
            #[must_use]
            pub fn from_code(code: i32) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

pub(crate) use coded;

/// Declares an enum of the values that one string argument can name.
/// Decoding ignores case, as scsynth does, and takes the other names of a
/// value after `|`. Encoding writes the first name. serde takes only the
/// first name, as it is written.
macro_rules! named {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $text:literal $(| $alias:literal)*
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                #[cfg_attr(feature = "serde", serde(rename = $text))]
                $variant,
            )*
        }

        impl $name {
            /// The name that travels on the wire.
            #[must_use]
            pub fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $text,)*
                }
            }

            /// The value that `name` names, in any case.
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                $(
                    if name.eq_ignore_ascii_case($text) $(|| name.eq_ignore_ascii_case($alias))* {
                        return Some(Self::$variant);
                    }
                )*
                None
            }
        }
    };
}

pub(crate) use named;

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use rosc::OscType::{Blob, Float, Int, String as Text};

    use crate::command::buffer::{HeaderFormat, SampleFormat};
    use crate::command::buffer_fill::WaveFillFlags;
    use crate::command::top_level::{DumpMode, ErrorMode};
    use crate::error::DecodeErrorKind;
    use crate::reply::command_replies::BufferCommand;
    use crate::testing::decode_error;
    use crate::{AddAction, Command, OrderAction, Reply};

    /// Each value has its code, and no other code from -8 to 8 decodes.
    fn assert_codes<T: Copy + Debug + PartialEq>(
        codes: &[(T, i32)],
        code: fn(T) -> i32,
        from_code: fn(i32) -> Option<T>,
    ) {
        for &(value, wire) in codes {
            assert_eq!(code(value), wire, "{value:?}");
            assert_eq!(from_code(wire), Some(value), "{wire}");
        }
        for wire in -8..=8 {
            if !codes.iter().any(|&(_, known)| known == wire) {
                assert_eq!(from_code(wire), None, "{wire}");
            }
        }
    }

    /// Each value has its name, and decodes from it and from its other
    /// names, in any case.
    fn assert_names<T: Copy + Debug + PartialEq>(
        names: &[(T, &str, &[&str])],
        name: fn(T) -> &'static str,
        from_name: fn(&str) -> Option<T>,
    ) {
        for &(value, wire, others) in names {
            assert_eq!(name(value), wire, "{value:?}");
            for text in [wire].iter().chain(others) {
                assert_eq!(from_name(text), Some(value), "{text}");
                assert_eq!(from_name(&text.to_uppercase()), Some(value), "{text}");
            }
        }
    }

    #[test]
    fn every_coded_value_has_its_code_in_scsynth() {
        assert_codes(
            &[
                (AddAction::Head, 0),
                (AddAction::Tail, 1),
                (AddAction::Before, 2),
                (AddAction::After, 3),
                (AddAction::Replace, 4),
            ],
            AddAction::code,
            AddAction::from_code,
        );
        assert_codes(
            &[
                (OrderAction::Head, 0),
                (OrderAction::Tail, 1),
                (OrderAction::Before, 2),
                (OrderAction::After, 3),
            ],
            OrderAction::code,
            OrderAction::from_code,
        );
        assert_codes(
            &[
                (DumpMode::Off, 0),
                (DumpMode::Parsed, 1),
                (DumpMode::Hex, 2),
                (DumpMode::ParsedAndHex, 3),
            ],
            DumpMode::code,
            DumpMode::from_code,
        );
        assert_codes(
            &[
                (ErrorMode::Off, 0),
                (ErrorMode::On, 1),
                (ErrorMode::BundleOff, -1),
                (ErrorMode::BundleOn, -2),
            ],
            ErrorMode::code,
            ErrorMode::from_code,
        );
        let flags = |normalize, wavetable, clear| WaveFillFlags {
            normalize,
            wavetable,
            clear,
        };
        assert_codes(
            &[
                (flags(false, false, false), 0),
                (flags(true, false, false), 1),
                (flags(false, true, false), 2),
                (flags(true, true, false), 3),
                (flags(false, false, true), 4),
                (flags(true, false, true), 5),
                (flags(false, true, true), 6),
                (flags(true, true, true), 7),
            ],
            WaveFillFlags::bits,
            WaveFillFlags::from_bits,
        );
        for (command, address) in [
            (BufferCommand::BAlloc, "/b_alloc"),
            (BufferCommand::BAllocRead, "/b_allocRead"),
            (BufferCommand::BAllocReadChannel, "/b_allocReadChannel"),
            (BufferCommand::BRead, "/b_read"),
            (BufferCommand::BReadChannel, "/b_readChannel"),
            (BufferCommand::BWrite, "/b_write"),
            (BufferCommand::BFree, "/b_free"),
            (BufferCommand::BZero, "/b_zero"),
            (BufferCommand::BGen, "/b_gen"),
            (BufferCommand::BClose, "/b_close"),
        ] {
            assert_eq!(command.address(), address);
        }
    }

    #[test]
    fn every_format_has_its_names_in_scsynth() {
        assert_names(
            &[
                (HeaderFormat::Aiff, "aiff", &["aifc"]),
                (HeaderFormat::Wav, "wav", &["wave", "riff"]),
                (HeaderFormat::WavEx, "wavex", &[]),
                (HeaderFormat::Next, "next", &["sun"]),
                (HeaderFormat::Ircam, "ircam", &[]),
                (HeaderFormat::Raw, "raw", &[]),
                (HeaderFormat::Mat4, "mat4", &[]),
                (HeaderFormat::Mat5, "mat5", &[]),
                (HeaderFormat::Paf, "paf", &[]),
                (HeaderFormat::Svx, "svx", &[]),
                (HeaderFormat::Nist, "nist", &[]),
                (HeaderFormat::Voc, "voc", &[]),
                (HeaderFormat::W64, "w64", &[]),
                (HeaderFormat::Pvf, "pvf", &[]),
                (HeaderFormat::Xi, "xi", &[]),
                (HeaderFormat::Htk, "htk", &[]),
                (HeaderFormat::Sds, "sds", &[]),
                (HeaderFormat::Avr, "avr", &[]),
                (HeaderFormat::Sd2, "sd2", &[]),
                (HeaderFormat::Flac, "flac", &[]),
                (HeaderFormat::Ogg, "ogg", &[]),
                (HeaderFormat::Caf, "caf", &[]),
                (HeaderFormat::Rf64, "rf64", &[]),
                (HeaderFormat::Mpeg, "mpeg", &[]),
            ],
            HeaderFormat::name,
            HeaderFormat::from_name,
        );
        assert_names(
            &[
                (SampleFormat::Uint8, "uint8", &[]),
                (SampleFormat::Int8, "int8", &[]),
                (SampleFormat::Int16, "int16", &[]),
                (SampleFormat::Int24, "int24", &[]),
                (SampleFormat::Int32, "int32", &[]),
                (SampleFormat::Float, "float", &[]),
                (SampleFormat::Double, "double", &[]),
                (SampleFormat::Mulaw, "mulaw", &["ulaw"]),
                (SampleFormat::Alaw, "alaw", &[]),
                (SampleFormat::Vorbis, "vorbis", &[]),
                (SampleFormat::Opus, "opus", &[]),
                (SampleFormat::Mp1, "mp1", &[]),
                (SampleFormat::Mp2, "mp2", &[]),
                (SampleFormat::Mp3, "mp3", &[]),
            ],
            SampleFormat::name,
            SampleFormat::from_name,
        );
        assert_eq!(SampleFormat::from_name(""), None);
    }

    #[test]
    fn a_string_a_blob_and_a_double_take_no_other_type() {
        assert_eq!(
            decode_error::<Command>("/d_load", vec![Int(1)]),
            DecodeErrorKind::WrongType {
                index: 0,
                expected: "a string",
                found: 'i'
            }
        );
        assert_eq!(
            decode_error::<Command>("/d_recv", vec![Text("a".to_owned())]),
            DecodeErrorKind::WrongType {
                index: 0,
                expected: "a blob",
                found: 's'
            }
        );
        let mut status: Vec<_> = [1, 0, 0, 1, 0].map(Int).into();
        status.extend([Float(0.5), Float(1.5), Float(44_100.0)]);
        assert_eq!(
            decode_error::<Reply>("/status.reply", status),
            DecodeErrorKind::WrongType {
                index: 7,
                expected: "a double",
                found: 'f'
            }
        );
        assert_eq!(
            decode_error::<Command>("/sync", vec![Blob(vec![7])]),
            DecodeErrorKind::WrongType {
                index: 0,
                expected: "an int",
                found: 'b'
            }
        );
    }

    /// `address` without its slash, with `.` as `_`, a `_` before each
    /// capital that follows a lowercase letter, in capitals.
    fn upper_snake(address: &str) -> String {
        let mut name = String::new();
        let mut after_lowercase = false;
        for character in address.trim_start_matches('/').chars() {
            if character.is_ascii_uppercase() && after_lowercase {
                name.push('_');
            }
            after_lowercase = character.is_ascii_lowercase();
            name.push(match character {
                '.' => '_',
                other => other.to_ascii_uppercase(),
            });
        }
        name
    }

    #[test]
    fn every_constant_is_its_address_in_upper_snake_case() {
        assert_eq!(upper_snake("/g_queryTree.reply"), "G_QUERY_TREE_REPLY");
        assert_eq!(upper_snake("/dumpOSC"), "DUMP_OSC");
        for (names, addresses) in [
            (Command::ADDRESS_NAMES, Command::ADDRESSES),
            (Reply::ADDRESS_NAMES, Reply::ADDRESSES),
        ] {
            assert_eq!(names.len(), addresses.len());
            for (name, address) in names.iter().zip(addresses) {
                assert_eq!(*name, upper_snake(address), "{address}");
            }
        }
        assert_eq!(Command::S_NEW, "/s_new");
        assert_eq!(Reply::STATUS_REPLY, "/status.reply");
    }
}
