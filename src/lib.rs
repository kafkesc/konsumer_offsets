//! TODO more package doc
//!
//! # Parsing Kafka _entrails_
//!
//! Kafka runs on the JVM, so it's limited to what the JVM supports.
//!
//! This crate deals with these limitations, and the data it parses are a reflection
//! of what Java can indeed support. So, for example, in places where one would expect
//! an unsigned integer, like a string length or an ever increasing identifier, we use
//! the signed integers that Java supports.
//!
//! ## Java Primitive types in Rust
//!
//! Java uses a more limited set of [Primitive Types] for scalar values, compared to Rust.
//! Fortunately, Rust offering is large enough that is possible to map all Java types
//! to a Rust type:
//!
//! | Java      | Rust               |
//! |:---------:|:------------------:|
//! | `short`   | `i16`              |
//! | `int`     | `i32`              |
//! | `long`    | `i64`              |
//! | `float`   | `f32`              |
//! | `double`  | `f64`              |
//! | `boolean` | `u8`               |
//! | `char`    | `char(u16 as u32)` |
//!
//! And, additional limitation, Java only supported Big Endian encoding.
//!
//! ## Bespoke formats used by `__consumer_offsets`
//!
//! In addition to fields of one of the primitive types above,
//! messages of this internal topic also contain a few more structured types:
//! _string_ and _vector of bytes_.
//!
//! ### Type: _string_
//!
//! 2 bytes are used to store the string length as Java `short`,
//! followed by a UTF-8 encoded string of that length:
//!
//! ```text
//! 0        8       16     ...      8 x LENGTH
//! +-8-bits-+-8-bits-+-----------------------+
//! | 1 byte | 1 byte | ...LENGTH x bytes...  |
//! +-----------------------------------------+
//! |   i16: LENGTH   |      UTF8_STRING      |
//! +-----------------------------------------+
//! ```
//!
//! ### Type: _vector of bytes_
//!
//! 4 bytes are used to store the vector length as Java `int`,
//! followed by a sequence of bytes of that length:
//!
//! ```text
//! 0        8       16       24       32     ...      8 x LENGTH
//! +-8-bits-+-8-bits-+-8-bits-+-8-bits-+-----------------------+
//! | 1 byte | 1 byte | 1 byte | 1 byte | ...LENGTH x bytes...  |
//! +-----------------------------------------------------------+
//! |            i32: LENGTH            |     BYTES_VECTOR      |
//! +-----------------------------------------------------------+
//! ```
//!
//! [Primitive Types]: https://docs.oracle.com/javase/tutorial/java/nutsandbolts/datatypes.html

mod errors;
mod group_metadata;
mod offset_commit;
mod utils;

use errors::KonsumerOffsetsError;
use group_metadata::GroupMetadata;
use offset_commit::OffsetCommit;

use bytes_parser::BytesParser;

/*
Source: https://docs.oracle.com/javase/tutorial/java/nutsandbolts/datatypes.html
- java data is in Big Endian
- java short == i16
- java int == i32
- java long == i64
- java float == f32
- java double == f64
- java boolean == u8
- java char == u16, so for Rust it will need to be cast to u32
 */

pub(crate) const MSG_V0_OFFSET_COMMIT: i16 = 0;
pub(crate) const MSG_V1_OFFSET_COMMIT: i16 = 1;
pub(crate) const MSG_V2_OFFSET_COMMIT: i16 = 2;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum KonsumerOffsetsData {
    OffsetCommit(OffsetCommit),
    GroupMetadata(GroupMetadata),
}

pub fn parse(
    key: Option<&[u8]>,
    payload: Option<&[u8]>,
) -> Result<KonsumerOffsetsData, KonsumerOffsetsError> {
    // Throw error if a key is not provided: without we can't do much.
    if key.is_none() {
        return Err(KonsumerOffsetsError::CannotDetermineMessageVersionWithoutKey);
    }

    // In no payload is provided, this is a tombstone record.
    let is_tombstone = payload.is_none();

    let mut key_parser = BytesParser::from(key.unwrap());

    match key_parser.parse_i16() {
        Ok(message_version) => match message_version {
            MSG_V0_OFFSET_COMMIT..=MSG_V1_OFFSET_COMMIT => {
                let mut offset_commit = OffsetCommit::new(message_version);

                offset_commit.parse_key_fields(&mut key_parser)?;

                if !is_tombstone {
                    let mut payload_parser = BytesParser::from(payload.unwrap());
                    offset_commit.parse_value_fields(&mut payload_parser)?;
                } else {
                    offset_commit.is_tombstone = true;
                }

                Ok(KonsumerOffsetsData::OffsetCommit(offset_commit))
            }
            MSG_V2_OFFSET_COMMIT => {
                let mut group_metadata = GroupMetadata::new(message_version);

                group_metadata.parse_key_fields(&mut key_parser)?;

                if !is_tombstone {
                    let mut payload_parser = BytesParser::from(payload.unwrap());
                    group_metadata.parse_value_fields(&mut payload_parser)?;
                } else {
                    group_metadata.is_tombstone = true;
                }

                Ok(KonsumerOffsetsData::GroupMetadata(group_metadata))
            }
            _ => Err(KonsumerOffsetsError::UnsupportedMessageVersion(
                message_version,
            )),
        },
        Err(e) => Err(KonsumerOffsetsError::ByteParsingError(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn it_works() {
        // let kp = Path::new("fixtures/tests/01-offset_commit_key");
        // let key_bytes = fs::read(kp).unwrap();
        // let pp = Path::new("fixtures/tests/01-offset_commit_payload");
        // let payload_bytes = fs::read(pp).unwrap();
        //
        // let _ = parse(Some(key_bytes.as_slice()), Some(payload_bytes.as_slice()));

        let kp = Path::new("fixtures/tests/02-group_metadata_key");
        let key_bytes = fs::read(kp).unwrap();
        let pp = Path::new("fixtures/tests/02-group_metadata_payload");
        let payload_bytes = fs::read(pp).unwrap();

        let x = parse(Some(key_bytes.as_slice()), Some(payload_bytes.as_slice()));
        println!("{:#?}", x);
    }
}
