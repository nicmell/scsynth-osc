//! The strict reading and the canonical writing of arguments, and what the
//! enums of commands and replies share.

use rosc::{OscMessage, OscType};

use crate::error::{DecodeError, DecodeErrorKind};

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

/// A cursor over the arguments of one message. It takes each argument only
/// in the type that the protocol gives it.
pub(crate) struct Reader<'a> {
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

    /// The argument at `index` has a value that the protocol does not take.
    pub(crate) fn invalid(&self, index: usize, reason: impl Into<String>) -> DecodeError {
        self.error(DecodeErrorKind::Invalid {
            index,
            reason: reason.into(),
        })
    }

    /// The position of the next argument.
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn remains(&self) -> bool {
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
    pub(crate) fn wrong(&self, value: &OscType, expected: &'static str) -> DecodeError {
        self.error(DecodeErrorKind::WrongType {
            index: self.index - 1,
            expected,
            found: tag(value),
        })
    }

    pub(crate) fn int(&mut self) -> Result<i32, DecodeError> {
        match self.any("an int")? {
            OscType::Int(value) => Ok(*value),
            other => Err(self.wrong(other, "an int")),
        }
    }

    pub(crate) fn float(&mut self) -> Result<f32, DecodeError> {
        match self.any("a float")? {
            OscType::Float(value) => Ok(*value),
            other => Err(self.wrong(other, "a float")),
        }
    }

    pub(crate) fn double(&mut self) -> Result<f64, DecodeError> {
        match self.any("a double")? {
            OscType::Double(value) => Ok(*value),
            other => Err(self.wrong(other, "a double")),
        }
    }

    pub(crate) fn string(&mut self) -> Result<String, DecodeError> {
        match self.any("a string")? {
            OscType::String(value) => Ok(value.clone()),
            other => Err(self.wrong(other, "a string")),
        }
    }

    pub(crate) fn blob(&mut self) -> Result<Vec<u8>, DecodeError> {
        match self.any("a blob")? {
            OscType::Blob(value) => Ok(value.clone()),
            other => Err(self.wrong(other, "a blob")),
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

/// The arguments of one message, as they are written.
#[derive(Default)]
pub(crate) struct Writer {
    args: Vec<OscType>,
}

impl Writer {
    pub(crate) fn int(&mut self, value: i32) {
        self.args.push(OscType::Int(value));
    }

    pub(crate) fn float(&mut self, value: f32) {
        self.args.push(OscType::Float(value));
    }

    pub(crate) fn double(&mut self, value: f64) {
        self.args.push(OscType::Double(value));
    }

    pub(crate) fn string(&mut self, value: &str) {
        self.args.push(OscType::String(value.to_owned()));
    }

    pub(crate) fn blob(&mut self, value: Vec<u8>) {
        self.args.push(OscType::Blob(value));
    }

    pub(crate) fn push(&mut self, value: OscType) {
        self.args.push(value);
    }

    pub(crate) fn finish(self) -> Vec<OscType> {
        self.args
    }
}

/// The arguments of a message: how they are written, and how they are read.
pub(crate) trait Payload: Sized {
    fn write(&self, writer: &mut Writer);

    fn read(reader: &mut Reader<'_>) -> Result<Self, DecodeError>;
}

/// A command or a reply, as one OSC message. Sealed: it is public only so
/// that [`crate::Packet`] can name it.
pub trait Message: Sized {
    #[doc(hidden)]
    fn to_osc(&self) -> OscMessage;

    #[doc(hidden)]
    fn from_osc(message: &OscMessage) -> Result<Self, DecodeError>;
}

/// Declares the enum of the commands or of the replies: one variant per
/// address, a unit variant when the message has no arguments, and the
/// dispatch by address in both directions.
macro_rules! messages {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $address:literal => $variant:ident $(($payload:ty))?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(tag = "address", content = "args"))]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                #[cfg_attr(feature = "serde", serde(rename = $address))]
                $variant $(($payload))?,
            )*
        }

        impl $name {
            /// The OSC address of the message.
            pub fn address(&self) -> &'static str {
                match self {
                    $(Self::$variant { .. } => $address,)*
                }
            }

            /// The bytes of the message, as one OSC message.
            pub fn encode(&self) -> Vec<u8> {
                $crate::packet::encode_message(<Self as $crate::codec::Message>::to_osc(self))
            }

            /// The message in `bytes`, which must hold exactly one OSC
            /// message.
            pub fn decode(bytes: &[u8]) -> Result<Self, $crate::DecodeError> {
                let message = $crate::packet::decode_message(bytes)?;
                <Self as $crate::codec::Message>::from_osc(&message)
            }
        }

        impl $crate::codec::Message for $name {
            fn to_osc(&self) -> rosc::OscMessage {
                let mut writer = $crate::codec::Writer::default();
                match self {
                    $(
                        Self::$variant $(($crate::codec::messages!(@bind payload $payload)))? => {
                            $($crate::codec::messages!(@write writer payload $payload);)?
                        }
                    )*
                }
                rosc::OscMessage {
                    addr: self.address().to_owned(),
                    args: writer.finish(),
                }
            }

            fn from_osc(message: &rosc::OscMessage) -> Result<Self, $crate::DecodeError> {
                let mut reader = $crate::codec::Reader::new(&message.addr, &message.args);
                let value = match message.addr.as_str() {
                    $($address => $crate::codec::messages!(@read reader $variant $($payload)?),)*
                    _ => return Err(reader.error($crate::DecodeErrorKind::UnknownAddress)),
                };
                reader.finish()?;
                Ok(value)
            }
        }
    };
    (@bind $binding:ident $payload:ty) => {
        $binding
    };
    (@write $writer:ident $binding:ident $payload:ty) => {
        <$payload as $crate::codec::Payload>::write($binding, &mut $writer)
    };
    (@read $reader:ident $variant:ident) => {
        Self::$variant
    };
    (@read $reader:ident $variant:ident $payload:ty) => {
        Self::$variant(<$payload as $crate::codec::Payload>::read(&mut $reader)?)
    };
}

pub(crate) use messages;

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
            pub fn code(self) -> i32 {
                match self {
                    $(Self::$variant => $code,)*
                }
            }

            /// The value of `code`, when the protocol knows it.
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
