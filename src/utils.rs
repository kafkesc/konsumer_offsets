use bytes_parser::BytesParser;

use crate::errors::KonsumerOffsetsError;

/// A `__consumer_offsets` specific parser for [`String`].
///
/// This expects strings coming from the `__consumer_offsets` Kafka internal topic,
/// to be encoded as: an `i16` representing "amount of bytes", followed by
/// an `&str` of that length.
///
/// The length is encoded in `i16`: Kafka uses a Java Short, that is
/// a signed 16-bit integer, encoded in Big-Endian. See Java
/// [Primitive Data Types](https://docs.oracle.com/javase/tutorial/java/nutsandbolts/datatypes.html)
/// for details.
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
/// This expects a vector of bytes coming from the `__consumer_offsets` Kafka internal topic,
/// to be encoded as: an `i32` representing "amount of bytes", followed by
/// a `[u8]` of that length.
///
/// The length is encoded in `i32`: Kafka uses a Java Integer, that is
/// a signed 32-bit integer, encoded in Big-Endian. See Java
/// [Primitive Data Types](https://docs.oracle.com/javase/tutorial/java/nutsandbolts/datatypes.html)
/// for details.
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
