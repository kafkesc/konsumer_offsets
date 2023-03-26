use bytes_parser::BytesParser;

#[cfg(feature = "ts_chrono")]
use chrono::NaiveDateTime;
#[cfg(feature = "ts_time")]
use time::PrimitiveDateTime;

use crate::errors::{KonsumerOffsetsError, KonsumerOffsetsError::UnsupportedOffsetCommitSchema};
#[cfg(feature = "ts_chrono")]
use crate::utils::parse_chrono_naive_datetime;
use crate::utils::{parse_i16, parse_i32, parse_i64, parse_str};

/// Offset that a Kafka [Consumer] of a Group has reached when consuming a Partition of a Topic.
///
/// This is produced by the [Group Coordinator] when handling an `OffsetCommitRequest`
/// by a [Consumer], hence realizing [Offset Tracking].
///
/// This information has many uses, but the important one is to "maintain state" for the Consumer:
/// if a topic partition is reassigned to another [Consumer] in the same group, the new assignee
/// receives this information and knows where to resume consumption from.
///
/// Kafka uses code generation to materialise [`OffsetCommit`] into Java code,
/// and this is composed of 2 json definitions, that at compile time get turned into Java Classes:
/// [`OffsetCommitKey`] and [`OffsetCommitValue`].
///
/// **Note:** As this data is parsed from a message, each field is marked with **`(KEY)`**
/// or **`(PAYLOAD)`**, depending to what part of the message they were parsed from.
///
/// [Consumer]: https://kafka.apache.org/documentation/#theconsumer
/// [`OffsetCommitKey`]: https://github.com/apache/kafka/blob/trunk/core/src/main/resources/common/message/OffsetCommitKey.json
/// [`OffsetCommitValue`]: https://github.com/apache/kafka/blob/trunk/core/src/main/resources/common/message/OffsetCommitValue.json
/// [Group Coordinator]: https://github.com/apache/kafka/blob/trunk/core/src/main/scala/kafka/coordinator/group/GroupCoordinator.scala
/// [Offset Tracking]: https://kafka.apache.org/documentation/#impl_offsettracking
///
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct OffsetCommit {
    /// **`(KEY)`** First 2-bytes integers in the original `__consumer_offsets`, identifying this data type.
    ///
    /// This controls the bespoke binary parser behaviour.
    pub message_version: i16,

    /// **`(KEY)`** Group the Consumer belongs to.
    pub group: String,

    /// **`(KEY)`** Topic the Consumer subscribes to.
    pub topic: String,

    /// **`(KEY)`** Partition the Consumer is assignee of.
    pub partition: i32,

    /// **`(PAYLOAD)`** Is this from a _tombstone_ message?
    ///
    /// If this is `true`, this struct doesn't represent any offset, but the removal
    /// of this specific key (i.e. `(group,topic,partition)` tuple) from `__consumer_offsets`.
    ///
    /// If you are tracking this data, this can be used as a "can be removed" signal:
    /// likely all consumers of this particular group are gone, and something explicitly
    /// removed their offset tracking information.
    ///
    /// The removal follows the [Log Compaction] rules of Kafka.
    ///
    /// [Log Compaction]: https://kafka.apache.org/documentation/#compaction
    pub is_tombstone: bool,

    /// **`(PAYLOAD)`** Informs the parser of what data and in which format, the rest of the payload contains.
    ///
    /// This controls the bespoke binary parser behaviour.
    pub schema_version: i16,

    /// **`(PAYLOAD)`** Offset that a [`OffsetCommit::group`] has reached when consuming [`OffsetCommit::partition`] of [`OffsetCommit::topic`].
    pub offset: i64,

    /// **`(PAYLOAD)`** Leader epoch of the previously consumed record (if one is known).
    ///
    /// If a leader epoch is not known, this field will be `-1`. It can be used to
    /// filter out-of-date information, in transitional periods when leader is changing.
    pub leader_epoch: i32,

    /// **`(PAYLOAD)`** [Consumer] set, optional metadata.
    ///
    /// Default consumer behaviour is to leave this empty.
    ///
    /// [Consumer]: https://github.com/apache/kafka/tree/trunk/clients/src/main/java/org/apache/kafka/clients/consumer
    pub metadata: String,

    /// **`(PAYLOAD)`** Timestamp of when the offset was committed by the consumer.
    ///
    /// This timestamp is produced to `__consumer_offsets` by the [Group Coordinator]:
    /// to interpret it correctly, its important to know its timezone.
    ///
    /// **NOTE:** The type of this field is controlled by the `ts_*` feature flags.
    ///
    /// [Group Coordinator]: https://github.com/apache/kafka/blob/trunk/core/src/main/scala/kafka/coordinator/group/GroupCoordinator.scala
    #[cfg(feature = "ts_int")]
    pub commit_timestamp: i64,
    #[cfg(feature = "ts_chrono")]
    pub commit_timestamp: NaiveDateTime,
    #[cfg(feature = "ts_time")]
    pub commit_timestamp: PrimitiveDateTime,

    /// **`(PAYLOAD)`** Timestamp of when the offset will fall from topic retention.
    ///
    /// **NOTE:** The type of this field is controlled by the `ts_*` feature flags.
    ///
    /// **WARNING:** this is no longer supported, and in modern versions of Kafka it will
    /// be set to `-1`. It's here for parse completeness.
    #[cfg(feature = "ts_int")]
    pub expire_timestamp: i64,
    #[cfg(feature = "ts_chrono")]
    pub expire_timestamp: NaiveDateTime,
    #[cfg(feature = "ts_time")]
    pub expire_timestamp: PrimitiveDateTime,
}

impl OffsetCommit {
    /// Create [`Self`] from the key part of the message.
    ///
    /// The fields marked with **`(KEY)`** are parsed here.
    ///
    /// This is based on the generated `kafka.internals.generated.OffsetCommitKey#read` method.
    pub(crate) fn try_from(parser: &mut BytesParser, message_version: i16) -> Result<Self, KonsumerOffsetsError> {
        Ok(OffsetCommit {
            message_version,
            group: parse_str(parser)?,
            topic: parse_str(parser)?,
            partition: parse_i32(parser)?,
            is_tombstone: true,
            ..Default::default()
        })
    }

    /// Augment [`Self`] from data in the payload part of the message.
    ///
    /// The fields marked with **`(PAYLOAD)`** are parsed here.
    ///
    /// This is based on the generated `kafka.internals.generated.OffsetCommitValue#read` method.
    pub(crate) fn parse_payload(&mut self, parser: &mut BytesParser) -> Result<(), KonsumerOffsetsError> {
        self.is_tombstone = false;

        self.schema_version = parse_i16(parser)?;
        if !(0..=3).contains(&self.schema_version) {
            return Err(UnsupportedOffsetCommitSchema(self.schema_version));
        }

        self.offset = parse_i64(parser)?;

        self.leader_epoch = if self.schema_version >= 3 {
            parse_i32(parser)?
        } else {
            -1
        };

        self.metadata = parse_str(parser)?;

        #[cfg(feature = "ts_int")]
        {
            self.commit_timestamp = parse_i64(parser)?;
        }

        #[cfg(feature = "ts_chrono")]
        {
            self.commit_timestamp = parse_chrono_naive_datetime(parser)?;
        }

        // TODO Implement
        #[cfg(feature = "ts_time")]
        {
            self.commit_timestamp = PrimitiveDateTime::default();
        }

        self.expire_timestamp = if self.schema_version == 1 {
            #[cfg(feature = "ts_int")]
            {
                parse_i64(parser)?
            }

            #[cfg(feature = "ts_chrono")]
            {
                parse_chrono_naive_datetime(parser)?
            }

            #[cfg(feature = "ts_time")]
            {
                // TODO Implement:
                //   PrimitiveDateTime::try_from(cts_ms)?
            }
        } else {
            #[cfg(feature = "ts_int")]
            {
                -1
            }

            #[cfg(feature = "ts_chrono")]
            {
                NaiveDateTime::default()
            }

            #[cfg(feature = "ts_time")]
            {
                // TODO Implement:
                //   PrimitiveDateTime::try_from(cts_ms)?
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::is_thread_safe;
    use crate::OffsetCommit;

    #[test]
    fn test_types_thread_safety() {
        is_thread_safe::<OffsetCommit>();
    }
}
