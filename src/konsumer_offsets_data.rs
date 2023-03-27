use bytes_parser::BytesParser;

use crate::errors::KonsumerOffsetsError;
use crate::group_metadata::GroupMetadata;
use crate::offset_commit::OffsetCommit;

const MSG_V0_OFFSET_COMMIT: i16 = 0;
const MSG_V1_OFFSET_COMMIT: i16 = 1;
const MSG_V2_GROUP_METADATA: i16 = 2;

/// Possible types of data stored in `__consumer_offsets` topic.
///
/// This enum is a rust-idiomatic way to handle the fact that the messages read from
/// `__consumer_offsets` can be of different type. Ideally Kafka could have used 2 different
/// topics, but it doesn't so... here we are.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum KonsumerOffsetsData {
    /// Variant that wraps an [`OffsetCommit`] struct instance.
    OffsetCommit(OffsetCommit),

    /// Variant that wraps a [`GroupMetadata`] struct instance.
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
    pub fn try_from_bytes(
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

    /// Same as [`Self::try_from_bytes`], but handling input as [`Option<Vec<u8>>`].
    pub fn try_from_bytes_vec(
        key: Option<Vec<u8>>,
        payload: Option<Vec<u8>>,
    ) -> Result<KonsumerOffsetsData, KonsumerOffsetsError> {
        Self::try_from_bytes(key.as_deref(), payload.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use rstest::rstest;

    use super::*;
    use crate::utils::is_thread_safe;

    #[rstest]
    #[case(01)]
    #[case(02)]
    #[case(03)]
    #[case(04)]
    #[case(05)]
    fn from_offset_commit(#[case] fixture_id: u16) {
        #[cfg(feature = "ts_int")]
        let (key_bytes, payload_bytes, fmt_string) = read_offset_commit_fixture(fixture_id, "ts_int");
        #[cfg(feature = "ts_chrono")]
        let (key_bytes, payload_bytes, fmt_string) = read_offset_commit_fixture(fixture_id, "ts_chrono");
        #[cfg(feature = "ts_time")]
        let (key_bytes, payload_bytes, fmt_string) = read_offset_commit_fixture(fixture_id, "ts_time");

        let konsumer_offsets_data =
            KonsumerOffsetsData::try_from_bytes(Some(key_bytes.as_slice()), Some(payload_bytes.as_slice()));
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
        #[cfg(feature = "ts_int")]
        let (key_bytes, payload_bytes, fmt_string) = read_group_metadata_fixture(fixture_id, "ts_int");
        #[cfg(feature = "ts_chrono")]
        let (key_bytes, payload_bytes, fmt_string) = read_group_metadata_fixture(fixture_id, "ts_chrono");
        #[cfg(feature = "ts_time")]
        let (key_bytes, payload_bytes, fmt_string) = read_group_metadata_fixture(fixture_id, "ts_time");

        let konsumer_offsets_data =
            KonsumerOffsetsData::try_from_bytes(Some(key_bytes.as_slice()), Some(payload_bytes.as_slice()));
        assert!(konsumer_offsets_data.is_ok());
        match konsumer_offsets_data.unwrap() {
            KonsumerOffsetsData::GroupMetadata(gm) => {
                // println!("{:#?}", gm);
                assert_eq!(format!("{:#?}", gm), fmt_string);
            },
            _ => panic!("Returned wrong enum value!"),
        }
    }

    fn read_offset_commit_fixture(fixture_id: u16, ts_feature: &str) -> (Vec<u8>, Vec<u8>, String) {
        read_fixture("offset_commit", fixture_id, ts_feature)
    }

    fn read_group_metadata_fixture(fixture_id: u16, ts_feature: &str) -> (Vec<u8>, Vec<u8>, String) {
        read_fixture("group_metadata", fixture_id, ts_feature)
    }

    fn read_fixture(fixture_name: &str, fixture_id: u16, ts_feature: &str) -> (Vec<u8>, Vec<u8>, String) {
        let k = format!("fixtures/tests/{fixture_name}/{fixture_id:02}.key");
        let key_path = Path::new(k.as_str());
        let p = format!("fixtures/tests/{fixture_name}/{fixture_id:02}.payload");
        let payload_path = Path::new(p.as_str());
        let f = format!("fixtures/tests/{fixture_name}/{fixture_id:02}.{ts_feature}.fmt");
        let fmt_path = Path::new(f.as_str());
        assert!(key_path.exists());
        assert!(payload_path.exists());
        assert!(fmt_path.exists());

        (fs::read(key_path).unwrap(), fs::read(payload_path).unwrap(), fs::read_to_string(fmt_path).unwrap())
    }

    #[test]
    fn test_types_thread_safety() {
        is_thread_safe::<KonsumerOffsetsData>();
    }
}
