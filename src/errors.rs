use bytes_parser::BytesParserError;
use thiserror::Error;

/// Errors variants that can be encountered when parsing of `__consumer_offsets` messages.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum KonsumerOffsetsError {
    /// The message contains no key, making parsing of the rest of the message impossible.
    ///
    /// The message key contains information to determine the version of the message, on which
    /// the rest of the parsing logic depends.
    #[error("Cannot parse message without its key: unable to determine version")]
    MessageKeyMissing,

    /// An error occurred during parsing.
    #[error("Failure while parsing bytes: {0}")]
    ByteParsingError(#[source] BytesParserError),

    #[cfg(feature = "ts_chrono")]
    /// An error occurred when parsing milliseconds to [`chrono::NaiveDateTime`].
    #[error("Failed to parse milliseconds into chrono::NaiveDateTime: {0}")]
    ChronoNaiveDateTimeParsingError(i64),

    /// Message key refers to a version format which this crate doesn't currently support.
    #[error("Encountered a not (yet) supported message version: {0}")]
    UnsupportedMessageVersion(i16),

    /// Message payload for an [`crate::offset_commit::OffsetCommit`] refers to a schema format which this crate doesn't currently support.
    #[error("Encountered a not (yet) supported schema version for offset commit: {0}")]
    UnsupportedOffsetCommitSchema(i16),

    /// Message payload for a [`crate::group_metadata::GroupMetadata`] refers to a schema format which this crate doesn't currently support.
    #[error("Encountered a not (yet) supported schema version for group metadata: {0}")]
    UnsupportedGroupMetadataSchema(i16),

    /// Message payload for a [`crate::group_metadata::ConsumerProtocolSubscription`] refers to a version format which this crate doesn't currently support.
    #[error("Encountered a not (yet) supported consumer protocol subscription version: {0}")]
    UnsupportedConsumerProtocolSubscriptionVersion(i16),

    /// Message payload for a [`crate::group_metadata::ConsumerProtocolAssignment`] refers to a version format which this crate doesn't currently support.
    #[error("Encountered a not (yet) supported consumer protocol assignment version: {0}")]
    UnsupportedConsumerProtocolAssignmentVersion(i16),

    /// A value of a given type cannot be parsed for a specific version of another type.
    #[error("Unable to parse {0} for version {1} of {2}")]
    UnableToParseForVersion(String, i16, String),
}

#[cfg(test)]
mod tests {
    use crate::utils::is_thread_safe;
    use crate::KonsumerOffsetsError;

    #[test]
    fn test_types_thread_safety() {
        is_thread_safe::<KonsumerOffsetsError>();
    }
}
