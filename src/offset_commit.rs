use bytes_parser::BytesParser;

use crate::errors::KonsumerOffsetsError;
use crate::utils::{parse_i16, parse_i32, parse_i64, parse_str};

/// TODO doc
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct OffsetCommit {
    pub message_version: i16,
    pub group: String,
    pub topic: String,
    pub partition: i32,

    pub is_tombstone: bool,

    pub schema_version: i16,
    pub offset: i64,
    pub leader_epoch: i32,
    pub metadata: String,
    pub commit_timestamp: i64,
    pub expire_timestamp: i64,
}

impl OffsetCommit {
    /// TODO doc
    ///
    /// NOTE: This is based on `kafka.internals.generated.OffsetCommitKey#read` method..
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

    /// TODO doc
    ///
    /// NOTE: This is based on `kafka.internals.generated.OffsetCommitValue#read` method.
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
