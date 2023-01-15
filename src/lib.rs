//! TODO more package doc
//!
//! # Parsing Kafka _entrails_
//!
//! Kafka runs on the JVM, so it's limited to what the JVM supports.
//!
//! This crate deals with these limitations, and the data it parses are a reflection
//! of what Java can indeed support. So, for example, in places where one would expect
//! an unsigned integer, like a string length or an ever increasing identifier, we use
//! the signed integers that Java supports.
//!
//! ## Java Primitive types in Rust
//!
//! Java uses a more limited set of [Primitive Types] for scalar values, compared to Rust.
//! Fortunately, Rust offering is large enough that is possible to map all Java types
//! to a Rust type:
//!
//! | Java      | Rust               |
//! |:---------:|:------------------:|
//! | `short`   | `i16`              |
//! | `int`     | `i32`              |
//! | `long`    | `i64`              |
//! | `float`   | `f32`              |
//! | `double`  | `f64`              |
//! | `boolean` | `u8`               |
//! | `char`    | `char(u16 as u32)` |
//!
//! And, additional limitation, Java only supported Big Endian encoding.
//!
//! ## Bespoke formats used by `__consumer_offsets`
//!
//! In addition to fields of one of the primitive types above,
//! messages of this internal topic also contain a few more structured types:
//! _string_ and _vector of bytes_.
//!
//! ### Type: _string_
//!
//! 2 bytes are used to store the string length as Java `short`,
//! followed by a UTF-8 encoded string of that length:
//!
//! ```text
//! 0        8       16     ...      8 x LENGTH
//! +-8-bits-+-8-bits-+-----------------------+
//! | 1 byte | 1 byte | ...LENGTH x bytes...  |
//! +-----------------------------------------+
//! |   i16: LENGTH   |      UTF8_STRING      |
//! +-----------------------------------------+
//! ```
//!
//! ### Type: _vector of bytes_
//!
//! 4 bytes are used to store the vector length as Java `int`,
//! followed by a sequence of bytes of that length:
//!
//! ```text
//! 0        8       16       24       32     ...      8 x LENGTH
//! +-8-bits-+-8-bits-+-8-bits-+-8-bits-+-----------------------+
//! | 1 byte | 1 byte | 1 byte | 1 byte | ...LENGTH x bytes...  |
//! +-----------------------------------------------------------+
//! |            i32: LENGTH            |     BYTES_VECTOR      |
//! +-----------------------------------------------------------+
//! ```
//!
//! [Primitive Types]: https://docs.oracle.com/javase/tutorial/java/nutsandbolts/datatypes.html

mod errors;
mod group_metadata;
mod konsumer_offsets_data;
mod offset_commit;
mod utils;

pub use errors::*;
pub use group_metadata::*;
pub use konsumer_offsets_data::*;
pub use offset_commit::*;
