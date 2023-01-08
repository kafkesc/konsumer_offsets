use bytes_parser::BytesParserError;
use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum KonsumerOffsetsError {
    #[error("Cannot parse message without its key: unable to determine version")]
    CannotDetermineMessageVersionWithoutKey,

    #[error("Failure while parsing bytes: {0}")]
    ByteParsingError(#[source] BytesParserError),

    #[error("Encountered a not (yet) supported message version: {0}")]
    UnsupportedMessageVersion(i16),

    #[error("Encountered a not (yet) supported schema version for offset commit: {0}")]
    UnsupportedOffsetCommitSchema(i16),

    #[error("Encountered a not (yet) supported schema version for group metadata: {0}")]
    UnsupportedGroupMetadataSchema(i16),

    #[error("Encountered a not (yet) supported consumer protocol subscription version: {0}")]
    UnsupportedConsumerProtocolSubscriptionVersion(i16),

    #[error("Encountered a not (yet) supported consumer protocol assignment version: {0}")]
    UnsupportedConsumerProtocolAssignmentVersion(i16),

    #[error("Unable to parse {0} for version {1} of {2}")]
    UnableToParseForVersion(String, i16, String),
}
