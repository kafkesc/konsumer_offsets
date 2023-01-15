use bytes_parser::BytesParser;

use crate::errors::KonsumerOffsetsError;
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
/// **Note:** As this data is parsed from a message, each field is marked with **(KEY)** or **(PAYLOAD)**,
/// depending to what part of the message they were parsed from.
///
/// [Consumer]: https://kafka.apache.org/documentation/#theconsumer
/// [`OffsetCommitKey`]: https://github.com/apache/kafka/blob/trunk/core/src/main/resources/common/message/OffsetCommitKey.json
/// [`OffsetCommitValue`]: https://github.com/apache/kafka/blob/trunk/core/src/main/resources/common/message/OffsetCommitValue.json
/// [Group Coordinator]: https://github.com/apache/kafka/blob/trunk/core/src/main/scala/kafka/coordinator/group/GroupCoordinator.scala
/// [Offset Tracking]: https://kafka.apache.org/documentation/#impl_offsettracking
///
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct OffsetCommit {
    /// **(KEY)** First 2-bytes integers in the original `__consumer_offsets`, identifying this data type.
    pub message_version: i16,

    /// **(KEY)** Group the Consumer belongs to.
    pub group: String,

    /// **(KEY)** Topic the Consumer subscribes to.
    pub topic: String,

    /// **(KEY)** Partition the Consumer is assignee of.
    pub partition: i32,

    /// **(PAYLOAD)** Is this from a _tombstone_ message?
    ///
    /// If this is `true`, this struct doesn't represent any offset, but the removal
    /// of this specific key (i.e. `(group,topic,partition)` tuple) from `__consumer_offsets`.
    ///
    /// If you are tracking this data, this can be used as a "can be removed" signal:
    /// likely all consumers of this particular group are gone, and something removed their
    /// offset tracking information.
    ///
    /// The removal follows the [Log Compaction] rules of Kafka.
    ///
    /// [Log Compaction]: https://kafka.apache.org/documentation/#compaction
    pub is_tombstone: bool,

    /// **(PAYLOAD)** Informs the parser of what data and in which format, the rest of the payload contains.
    pub schema_version: i16,

    /// **(PAYLOAD)** Offset that a [`OffsetCommit::group`] has reached when consuming [`OffsetCommit::partition`] of [`OffsetCommit::topic`].
    pub offset: i64,

    /// **(PAYLOAD)** Leader epoch of the previously consumed record (if one is known).
    ///
    /// If a leader epoch is not known, this field will be `-1`. It can be used to
    /// filter out-of-date information, in transitional periods when leader is changing.
    pub leader_epoch: i32,

    /// **(PAYLOAD)** [Consumer] set, optional metadata.
    ///
    /// Default consumer behaviour is to leave this empty.
    ///
    /// [Consumer]: https://github.com/apache/kafka/tree/trunk/clients/src/main/java/org/apache/kafka/clients/consumer
    pub metadata: String,

    /// **(PAYLOAD)** Timestamp of when the offset was committed by the consumer, in milliseconds.
    ///
    /// This timestamp is produced to `__consumer_offsets` by the group coordinator when handling
    /// an `OffsetCommitRequest`: to interpret it correctly, its important to know the
    /// timezone of the group coordinator broker.
    pub commit_timestamp: i64,

    /// **(PAYLOAD)** Timestamp of when the offset will fall from topic retention, in milliseconds.
    ///
    /// **WARNING:** this is no longer supported, and in modern versions of Kafka it will be set to
    /// `-1`. It's here for parse completeness.
    pub expire_timestamp: i64,
}

impl OffsetCommit {
    /// Create [`Self`] from the key part of the message.
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

    /// Finish creating [`Self`] from the payload part of the message.
    ///
    /// This is based on the generated `kafka.internals.generated.OffsetCommitValue#read` method.
    pub(crate) fn parse_payload(&mut self, parser: &mut BytesParser) -> Result<(), KonsumerOffsetsError> {
        self.is_tombstone = false;

        self.schema_version = parse_i16(parser)?;
        if !(0..=3).contains(&self.schema_version) {
            return Err(KonsumerOffsetsError::UnsupportedOffsetCommitSchema(self.schema_version));
        }

        self.offset = parse_i64(parser)?;

        self.leader_epoch = if self.schema_version >= 3 {
            parse_i32(parser)?
        } else {
            -1
        };

        self.metadata = parse_str(parser)?;

        self.commit_timestamp = parse_i64(parser)?;

        self.expire_timestamp = if self.schema_version == 1 {
            parse_i64(parser)?
        } else {
            -1
        };

        Ok(())
    }
}
