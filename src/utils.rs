use bytes_parser::BytesParser;

use crate::errors::KonsumerOffsetsError;

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

    parser
        .parse_str_utf8(group_strlen as usize)
        .map(|s| s.into())
        .map_err(KonsumerOffsetsError::ByteParsingError)
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

    let slice = parser
        .parse_slice(bytes_array_len as usize)
        .map_err(KonsumerOffsetsError::ByteParsingError)?;

    Ok(slice.to_vec())
}

/// Adapter for [`BytesParser::parse_i16`].
///
/// # Arguments
///
/// * `parser` - A [`BytesParser`] with its internal cursor pointing
///     at the beginning of the [`i16`] we want to parse.
pub(crate) fn parse_i16(parser: &mut BytesParser) -> Result<i16, KonsumerOffsetsError> {
    parser
        .parse_i16()
        .map_err(KonsumerOffsetsError::ByteParsingError)
}

/// Adapter for [`BytesParser::parse_i32`].
///
/// # Arguments
///
/// * `parser` - A [`BytesParser`] with its internal cursor pointing
///     at the beginning of the [`i32`] we want to parse.
pub(crate) fn parse_i32(parser: &mut BytesParser) -> Result<i32, KonsumerOffsetsError> {
    parser
        .parse_i32()
        .map_err(KonsumerOffsetsError::ByteParsingError)
}

/// Adapter for [`BytesParser::parse_i64`].
///
/// # Arguments
///
/// * `parser` - A [`BytesParser`] with its internal cursor pointing
///     at the beginning of the [`i64`] we want to parse.
pub(crate) fn parse_i64(parser: &mut BytesParser) -> Result<i64, KonsumerOffsetsError> {
    parser
        .parse_i64()
        .map_err(KonsumerOffsetsError::ByteParsingError)
}
