//! Why bytes are not a message of the protocol.

use std::fmt;

/// Why bytes do not decode as a command or a reply of scsynth.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodeError {
    address: String,
    kind: DecodeErrorKind,
}

/// What is wrong with the bytes.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum DecodeErrorKind {
    /// The bytes are not an OSC packet. The text is the reason that the OSC
    /// decoder gives.
    Osc(String),
    /// A bundle, where one message must be.
    Bundle,
    /// No message of the protocol has this address.
    UnknownAddress,
    /// The message ends before one of its arguments.
    Missing {
        /// The position of the missing argument.
        index: usize,
        /// What the argument must be.
        expected: &'static str,
    },
    /// An argument has another OSC type than the protocol gives it.
    WrongType {
        /// The position of the argument.
        index: usize,
        /// What the argument must be.
        expected: &'static str,
        /// The OSC type tag of the argument.
        found: char,
    },
    /// The message goes on after its last argument.
    Trailing {
        /// The position of the first argument too many.
        index: usize,
    },
    /// An argument has a value that the protocol does not take.
    Invalid {
        /// The position of the argument.
        index: usize,
        /// Why the value is refused.
        reason: String,
    },
    /// The completion message of an asynchronous command does not decode.
    Completion {
        /// The position of the completion message.
        index: usize,
        /// Why it does not decode.
        error: Box<DecodeError>,
    },
}

impl DecodeError {
    pub(crate) fn new(address: &str, kind: DecodeErrorKind) -> Self {
        Self {
            address: address.to_owned(),
            kind,
        }
    }

    /// The address of the message. It is empty when the bytes are not an OSC
    /// message.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// What is wrong.
    pub fn kind(&self) -> &DecodeErrorKind {
        &self.kind
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let address = if self.address.is_empty() {
            "the packet"
        } else {
            &self.address
        };
        match &self.kind {
            DecodeErrorKind::Osc(reason) => write!(f, "not an OSC packet: {reason}"),
            DecodeErrorKind::Bundle => write!(f, "a bundle where one message must be"),
            DecodeErrorKind::UnknownAddress => write!(f, "{address}: no such message"),
            DecodeErrorKind::Missing { index, expected } => {
                write!(
                    f,
                    "{address}: argument {index} is missing, it must be {expected}"
                )
            }
            DecodeErrorKind::WrongType {
                index,
                expected,
                found,
            } => write!(
                f,
                "{address}: argument {index} has the type '{found}', it must be {expected}"
            ),
            DecodeErrorKind::Trailing { index } => {
                write!(
                    f,
                    "{address}: the message goes on after its last argument, at {index}"
                )
            }
            DecodeErrorKind::Invalid { index, reason } => {
                write!(f, "{address}: argument {index}: {reason}")
            }
            DecodeErrorKind::Completion { index, error } => {
                write!(f, "{address}: the completion message at {index}: {error}")
            }
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            DecodeErrorKind::Completion { error, .. } => Some(error.as_ref()),
            _ => None,
        }
    }
}
