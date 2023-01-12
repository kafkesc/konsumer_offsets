use bytes_parser::BytesParser;
use std::any::type_name;

use crate::errors::KonsumerOffsetsError;
use crate::utils::{parse_i16, parse_i32, parse_i64, parse_str, parse_vec_bytes};

/// TODO doc
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct GroupMetadata {
    pub message_version: i16,
    pub group: String,

    pub is_tombstone: bool,

    pub schema_version: i16,
    pub protocol_type: String,
    pub generation: i32,
    pub protocol: String,
    pub leader: String,
    pub current_state_timestamp: i64,
    pub members: Vec<MemberMetadata>,
}

impl GroupMetadata {
    pub(crate) fn new(message_version: i16) -> GroupMetadata {
        GroupMetadata {
            message_version,
            ..Default::default()
        }
    }

    /// TODO doc
    ///
    /// NOTE: This is based on `kafka.internals.generated.GroupMetadataKey#read` method.
    pub(crate) fn parse_key_fields(
        &mut self,
        parser: &mut BytesParser,
    ) -> Result<(), KonsumerOffsetsError> {
        self.group = parse_str(parser)?;

        Ok(())
    }

    /// TODO doc
    ///
    /// NOTE: This is based on `kafka.internals.generated.GroupMetadataValue#read` method.
    pub(crate) fn parse_value_fields(
        &mut self,
        parser: &mut BytesParser,
    ) -> Result<(), KonsumerOffsetsError> {
        self.schema_version = parse_i16(parser)?;
        if !(0..=3).contains(&self.schema_version) {
            return Err(KonsumerOffsetsError::UnsupportedGroupMetadataSchema(
                self.schema_version,
            ));
        }

        self.protocol_type = parse_str(parser)?;

        self.generation = parse_i32(parser)?;

        self.protocol = parse_str(parser)?;

        self.leader = parse_str(parser)?;

        self.current_state_timestamp = if self.schema_version >= 2 {
            parse_i64(parser)?
        } else {
            -1
        };

        let members_len = parse_i32(parser)?;
        self.members = Vec::with_capacity(members_len as usize);
        for _ in 0..members_len {
            self.members
                .push(MemberMetadata::try_from(parser, self.schema_version)?);
        }

        Ok(())
    }
}

/// TODO doc
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct MemberMetadata {
    pub id: String,
    pub group_instance_id: String,
    pub client_id: String,
    pub client_host: String,
    pub rebalance_timeout: i32,
    pub session_timeout: i32,
    pub subscription: ConsumerProtocolSubscription,
    pub assignment: ConsumerProtocolAssignment,
}

impl MemberMetadata {
    /// NOTE `kafka.internals.generated.GroupMetadataValue.MemberMetadata#read` method.
    fn try_from(
        parser: &mut BytesParser,
        schema_version: i16,
    ) -> Result<Self, KonsumerOffsetsError> {
        let mut member = Self {
            id: parse_str(parser)?,
            ..Default::default()
        };

        if schema_version >= 3 {
            member.group_instance_id = parse_str(parser)?;
        }

        member.client_id = parse_str(parser)?;

        member.client_host = parse_str(parser)?;

        member.rebalance_timeout = if schema_version >= 1 {
            parse_i32(parser)?
        } else {
            0
        };

        member.session_timeout = parse_i32(parser)?;

        let subscription_bytes_len = parse_i32(parser)?;
        let mut subscription_parser = parser
            .from_slice(subscription_bytes_len as usize)
            .map_err(KonsumerOffsetsError::ByteParsingError)?;
        member.subscription = ConsumerProtocolSubscription::try_from(&mut subscription_parser)?;

        let assignment_bytes_len = parse_i32(parser)?;
        let mut assignment_parser = parser
            .from_slice(assignment_bytes_len as usize)
            .map_err(KonsumerOffsetsError::ByteParsingError)?;
        member.assignment = ConsumerProtocolAssignment::try_from(&mut assignment_parser)?;

        Ok(member)
    }
}

/// TODO doc
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct ConsumerProtocolSubscription {
    version: i16,
    subscribed_topics: Vec<String>,
    user_data: Vec<u8>,
    owned_topic_partitions: Vec<TopicPartitions>,
    generation_id: i32,
    rack_id: String,
}

impl<'a> TryFrom<&mut BytesParser<'a>> for ConsumerProtocolSubscription {
    type Error = KonsumerOffsetsError;

    /// TODO doc
    ///
    /// NOTE Based on `org.apache.kafka.common.message.ConsumerProtocolSubscription#read` method.
    fn try_from(parser: &mut BytesParser) -> Result<Self, Self::Error> {
        let mut subscription = Self {
            version: parse_i16(parser)?,
            ..Default::default()
        };

        if !(0..=3).contains(&subscription.version) {
            return Err(
                KonsumerOffsetsError::UnsupportedConsumerProtocolSubscriptionVersion(
                    subscription.version,
                ),
            );
        }

        let subscribed_topics_len = parse_i32(parser)?;
        if subscribed_topics_len > 0 {
            subscription.subscribed_topics = Vec::with_capacity(subscribed_topics_len as usize);
            for _ in 0..subscribed_topics_len {
                subscription.subscribed_topics.push(parse_str(parser)?);
            }
        }

        subscription.user_data = parse_vec_bytes(parser)?;

        if subscription.version >= 1 {
            let owned_topic_partitions_len = parse_i32(parser)?;
            if owned_topic_partitions_len > 0 {
                subscription.owned_topic_partitions =
                    Vec::with_capacity(owned_topic_partitions_len as usize);
                for _ in 0..owned_topic_partitions_len {
                    subscription
                        .owned_topic_partitions
                        .push(TopicPartitions::try_from(parser, subscription.version)?);
                }
            }
        }

        subscription.generation_id = if subscription.version >= 2 {
            parse_i32(parser)?
        } else {
            -1
        };

        if subscription.version >= 3 {
            subscription.rack_id = parse_str(parser)?;
        }

        Ok(subscription)
    }
}

/// Represents a collection of partitions belonging to a specific topic.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct TopicPartitions {
    topic: String,
    partitions: Vec<i32>,
}

impl TopicPartitions {
    /// Parse a [`TopicPartitions`] out of parser handling the [`ConsumerProtocolSubscription`] bytes.
    ///
    /// The logic of this method was reverse-engineered from the
    /// `org.apache.kafka.common.message.ConsumerProtocolSubscription.TopicPartition#read` method
    /// residing in the [Kafka codebase](https://github.com/apache/kafka).
    fn try_from(parser: &mut BytesParser, version: i16) -> Result<Self, KonsumerOffsetsError> {
        if version > 3 {
            return Err(KonsumerOffsetsError::UnableToParseForVersion(
                type_name::<TopicPartitions>().to_string(),
                version,
                type_name::<ConsumerProtocolSubscription>().to_string(),
            ));
        }

        let mut topic_partitions = Self {
            topic: parse_str(parser)?,
            ..Default::default()
        };

        let partitions_len = parse_i32(parser)?;
        if partitions_len > 0 {
            topic_partitions.partitions = Vec::with_capacity(partitions_len as usize);
            for _ in 0..partitions_len {
                topic_partitions.partitions.push(parse_i32(parser)?);
            }
        }

        Ok(topic_partitions)
    }
}

/// TODO doc
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct ConsumerProtocolAssignment {
    version: i16,
    assigned_topic_partitions: Vec<TopicPartitions>,
    user_data: Vec<u8>,
}

impl<'a> TryFrom<&mut BytesParser<'a>> for ConsumerProtocolAssignment {
    type Error = KonsumerOffsetsError;

    /// TODO doc
    ///
    /// NOTE Based on `org.apache.kafka.common.message.ConsumerProtocolAssignment#read` method.
    fn try_from(parser: &mut BytesParser) -> Result<Self, Self::Error> {
        let mut assignment = Self {
            version: parse_i16(parser)?,
            ..Default::default()
        };

        if !(0..=3).contains(&assignment.version) {
            return Err(
                KonsumerOffsetsError::UnsupportedConsumerProtocolAssignmentVersion(
                    assignment.version,
                ),
            );
        }

        let assigned_topic_partitions_len = parse_i32(parser)?;
        if assigned_topic_partitions_len > 0 {
            assignment.assigned_topic_partitions =
                Vec::with_capacity(assigned_topic_partitions_len as usize);
            for _ in 0..assigned_topic_partitions_len {
                assignment
                    .assigned_topic_partitions
                    .push(TopicPartitions::try_from(parser, assignment.version)?);
            }
        }

        assignment.user_data = parse_vec_bytes(parser)?;

        Ok(assignment)
    }
}
