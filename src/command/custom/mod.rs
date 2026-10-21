//! The commands of sc-app3's pump, which never reach scsynth, one module
//! per family. See
//! [The protocol of sc-app3's pump](crate#the-protocol-of-sc-app3s-pump).

mod groups;

pub use groups::{Plugin, Root};

use uuid::Uuid;

use crate::codec::{Reader, Writer};
use crate::error::DecodeError;

pub(crate) fn write_uuid(writer: &mut Writer, uuid: Uuid) {
    writer.string(&uuid.hyphenated().to_string());
}

/// A uuid, as the string that [`write_uuid`] writes.
pub(crate) fn read_uuid(reader: &mut Reader<'_>) -> Result<Uuid, DecodeError> {
    let index = reader.index();
    let text = reader.string()?;
    Uuid::try_parse(&text)
        .ok()
        .filter(|uuid| uuid.hyphenated().to_string() == text)
        .ok_or_else(|| {
            reader.invalid(
                index,
                format!("{text:?} is not a uuid in lowercase with hyphens"),
            )
        })
}
