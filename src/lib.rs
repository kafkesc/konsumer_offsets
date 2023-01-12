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

use bytes_parser::BytesParser;

pub use errors::KonsumerOffsetsError;
pub use group_metadata::*;
pub use offset_commit::*;

pub(crate) const MSG_V0_OFFSET_COMMIT: i16 = 0;
pub(crate) const MSG_V1_OFFSET_COMMIT: i16 = 1;
pub(crate) const MSG_V2_GROUP_METADATA: i16 = 2;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum KonsumerOffsetsData {
    OffsetCommit(OffsetCommit),
    GroupMetadata(GroupMetadata),
}

impl KonsumerOffsetsData {
    /// Parses the content of a messages (a.k.a. records) from the `__consumer_offsets`.
    ///
    /// `__consumer_offsets` it's a key internal Kafka topic, used by Kafka Consumers to
    /// keep track of the offset consumed so far, and also to coordinate their subscriptions
    /// and partition assignments.
    ///
    /// This method returns a [`KonsumerOffsetsData`]: an enum where each variant represents
    /// one of the possible messages that this topic carries.
    /// Refer to the documentation of each variant for details.
    ///
    /// **NOTE:** As Kafka messages have key and payload both optional,
    /// the signature reflects that. But messages of `__consumer_offsets`
    /// have at least a key set: if absent, the method will return an error.
    ///
    /// # Arguments
    ///
    /// * `key` - An [`Option`] of `&[u8]`: when set, it means the source Kafka message has a key.
    ///     If `key` is `None`, this function will return an error: it's likely that the message
    ///     it's not actually from the `__consumer_offsets` topic.
    /// * `payload` - An [`Option`] of `&[u8]`: when set, it means the source Kafka message has
    ///     a payload. If `payload` is `None`, the source Kafka message is a tombstone.
    pub fn try_from_message(
        key: Option<&[u8]>,
        payload: Option<&[u8]>,
    ) -> Result<KonsumerOffsetsData, KonsumerOffsetsError> {
        match key {
            // Throw error if a key is not provided: without we can't do much.
            None => Err(KonsumerOffsetsError::MessageKeyMissing),
            Some(key_bytes) => {
                let mut key_parser = BytesParser::from(key_bytes);
                match key_parser.parse_i16() {
                    Ok(message_version) => match message_version {
                        // Is it an `OffsetCommit`?
                        MSG_V0_OFFSET_COMMIT..=MSG_V1_OFFSET_COMMIT => {
                            let mut offset_commit = OffsetCommit::try_from(&mut key_parser, message_version)?;

                            // If there is a payload, parse it; otherwise, it's a tombstone.
                            if let Some(payload_bytes) = payload {
                                let mut payload_parser = BytesParser::from(payload_bytes);
                                offset_commit.parse_payload(&mut payload_parser)?;
                            }

                            Ok(KonsumerOffsetsData::OffsetCommit(offset_commit))
                        },
                        // Is it a `GroupMetadata`?
                        MSG_V2_GROUP_METADATA => {
                            let mut group_metadata = GroupMetadata::try_from(&mut key_parser, message_version)?;

                            // If there is a payload, parse it; otherwise, it's a tombstone.
                            if let Some(payload_bytes) = payload {
                                let mut payload_parser = BytesParser::from(payload_bytes);
                                group_metadata.parse_payload(&mut payload_parser)?;
                            }

                            Ok(KonsumerOffsetsData::GroupMetadata(group_metadata))
                        },
                        _ => Err(KonsumerOffsetsError::UnsupportedMessageVersion(message_version)),
                    },
                    Err(e) => Err(KonsumerOffsetsError::ByteParsingError(e)),
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(01)]
    #[case(02)]
    #[case(03)]
    #[case(04)]
    #[case(05)]
    fn from_offset_commit(#[case] fixture_id: u16) {
        let (key_bytes, payload_bytes, fmt_string) = read_offset_commit_fixture(fixture_id);

        let konsumer_offsets_data =
            KonsumerOffsetsData::try_from_message(Some(key_bytes.as_slice()), Some(payload_bytes.as_slice()));
        assert!(konsumer_offsets_data.is_ok());
        match konsumer_offsets_data.unwrap() {
            KonsumerOffsetsData::OffsetCommit(oc) => {
                // println!("{:#?}", oc);
                assert_eq!(format!("{:#?}", oc), fmt_string);
            },
            _ => panic!("Returned wrong enum value!"),
        }
    }

    #[rstest]
    #[case(01)]
    #[case(02)]
    #[case(03)]
    #[case(04)]
    #[case(05)]
    fn from_group_metadata(#[case] fixture_id: u16) {
        let (key_bytes, payload_bytes, fmt_string) = read_group_metadata_fixture(fixture_id);

        let konsumer_offsets_data =
            KonsumerOffsetsData::try_from_message(Some(key_bytes.as_slice()), Some(payload_bytes.as_slice()));
        assert!(konsumer_offsets_data.is_ok());
        match konsumer_offsets_data.unwrap() {
            KonsumerOffsetsData::GroupMetadata(gm) => {
                // println!("{:#?}", gm);
                assert_eq!(format!("{:#?}", gm), fmt_string);
            },
            _ => panic!("Returned wrong enum value!"),
        }
    }

    fn read_offset_commit_fixture(fixture_id: u16) -> (Vec<u8>, Vec<u8>, String) {
        read_fixture("offset_commit", fixture_id)
    }

    fn read_group_metadata_fixture(fixture_id: u16) -> (Vec<u8>, Vec<u8>, String) {
        read_fixture("group_metadata", fixture_id)
    }

    fn read_fixture(fixture_name: &str, fixture_id: u16) -> (Vec<u8>, Vec<u8>, String) {
        let k = format!("fixtures/tests/{fixture_name}/{fixture_id:02}.key");
        let key_path = Path::new(k.as_str());
        let p = format!("fixtures/tests/{fixture_name}/{fixture_id:02}.payload");
        let payload_path = Path::new(p.as_str());
        let f = format!("fixtures/tests/{fixture_name}/{fixture_id:02}.fmt");
        let fmt_path = Path::new(f.as_str());
        assert!(key_path.exists());
        assert!(payload_path.exists());
        assert!(fmt_path.exists());

        (fs::read(key_path).unwrap(), fs::read(payload_path).unwrap(), fs::read_to_string(fmt_path).unwrap())
    }
}
