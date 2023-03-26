use bytes_parser::BytesParser;

#[cfg(feature = "ts_chrono")]
use chrono::NaiveDateTime;

use crate::errors::KonsumerOffsetsError;
#[cfg(feature = "ts_chrono")]
use crate::errors::KonsumerOffsetsError::ChronoNaiveDateTimeParsingError;

/// A [`String`] parser, tailor-made for `__consumer_offsets` messages.
///
/// See the crate documentation for details about the format.
///
/// Returns a [`String::default`] if the parsed `i16` contains a negative value.
///
/// # Arguments
///
/// * `parser` - A [`BytesParser`] with its internal cursor pointing
///     at the beginning of the [`&str`] we want to parse.
pub(crate) fn parse_str(parser: &mut BytesParser) -> Result<String, KonsumerOffsetsError> {
    let group_strlen = parse_i16(parser)?;
    if group_strlen < 0 {
        return Ok(String::default());
    }

    parser.parse_str_utf8(group_strlen as usize).map(|s| s.into()).map_err(KonsumerOffsetsError::ByteParsingError)
}

/// A `__consumer_offsets` specific parser for [`Vec<u8>`].
///
/// See the crate documentation for details about the format.
///
/// # Arguments
///
/// * `parser` - A [`BytesParser`] with its internal cursor pointing
///     at the beginning of the [`Vec<u8>`] we want to parse.
pub(crate) fn parse_vec_bytes(parser: &mut BytesParser) -> Result<Vec<u8>, KonsumerOffsetsError> {
    let bytes_array_len = parse_i32(parser)?;

    let slice = parser.parse_slice(bytes_array_len as usize).map_err(KonsumerOffsetsError::ByteParsingError)?;

    Ok(slice.to_vec())
}

/// Adapter for [`BytesParser::parse_i16`].
///
/// # Arguments
///
/// * `parser` - A [`BytesParser`] with its internal cursor pointing
///     at the beginning of the [`i16`] we want to parse.
pub(crate) fn parse_i16(parser: &mut BytesParser) -> Result<i16, KonsumerOffsetsError> {
    parser.parse_i16().map_err(KonsumerOffsetsError::ByteParsingError)
}

/// Adapter for [`BytesParser::parse_i32`].
///
/// # Arguments
///
/// * `parser` - A [`BytesParser`] with its internal cursor pointing
///     at the beginning of the [`i32`] we want to parse.
pub(crate) fn parse_i32(parser: &mut BytesParser) -> Result<i32, KonsumerOffsetsError> {
    parser.parse_i32().map_err(KonsumerOffsetsError::ByteParsingError)
}

/// Adapter for [`BytesParser::parse_i64`].
///
/// # Arguments
///
/// * `parser` - A [`BytesParser`] with its internal cursor pointing
///     at the beginning of the [`i64`] we want to parse.
pub(crate) fn parse_i64(parser: &mut BytesParser) -> Result<i64, KonsumerOffsetsError> {
    parser.parse_i64().map_err(KonsumerOffsetsError::ByteParsingError)
}

#[cfg(feature = "ts_chrono")]
pub(crate) fn parse_chrono_naive_datetime(parser: &mut BytesParser) -> Result<NaiveDateTime, KonsumerOffsetsError> {
    let millis = parse_i64(parser)?;
    NaiveDateTime::from_timestamp_millis(millis).ok_or(ChronoNaiveDateTimeParsingError(millis))
}

/// Used in unit tests to verify type is Thread Safe and Async/Await Safe.
///
/// It enforces that the given type implements the following standard traits:
///
/// * `std::marker::Sized`: type has a constant size known at compile time
/// * `std::marker::Send`: type is safe to send to another thread
/// * `std::marker::Sync`: type is Sync if it is safe to share between threads;
///   type can be Sync if and only if a reference to it is Send
/// * `std::marker::Unpin`: type can be safely moved after pinning
///
/// # Examples
///
/// ```rust
/// use konsumer_offsets::GroupMetadata;
///
/// #[test]
/// fn test_types_thread_safety() {
///     is_thread_safe::<GroupMetadata>();
/// }
/// ```
#[cfg(test)]
pub(crate) fn is_thread_safe<T: Sized + Send + Sync + Unpin>() {}
