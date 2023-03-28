//! # Konsumer_Offsets: parse Kafka `__consumer_offsets`
//!
//! A library crate to parse the content of the [Kafka] [`__consumer_offsets`] internal topic.
//!
//! This was written by reverse-engineering the parsing logic used in [Kafka], looking at both
//! [Group Coordinator] Broker and [Consumer Client] source code.
//!
//! An **extra perk** of this crate, is that it serves also a didactic purpose: it attempts
//! to documenting really clearly what each struct and field represents, what it means in
//! the context of Kafka _"coordinated consumption of messages"_, and how it was parsed.
//!
//! ## Features
//!
//! Features:
//!
//! * `ts_int`: use `i64` to represent Unix Timestamps (conflicts with `ts_chrono` and `ts_time`)
//! * `ts_chrono`: use [`chrono::DateTime<Utc>`] to represent Unix Timestamps (conflicts with `ts_int` and `ts_time`)
//! * `ts_time`: use [`time::OffsetDateTime`] to represent Unix Timestamps (conflicts with `ts_chrono` and `ts_int`)
//! * `serde`: support serialization/deserialization for all types exposed by this crate via the [serde] crate
//!
//! Default features: `ts_int`.
//!
//! ## Before you dive in
//!
//! What this crate does, and what data it can parse, will make
//! sense only if you have read about how Kafka tracks Consumer Offsets. Especially, what
//! part the Consumer [Group Coordinator] play in that. A _great_ resource to learn (or refresh)
//! how the [Consumer Group Protocol] works, can be found on the [Confluent Developer] website.
//!
//! Also, as it will be referenced when documenting the data itself, it is important to know
//! that [`__consumer_offsets`] uses [Log Compaction]: this means that it follows all the compaction
//! rules, including the handling of _tombstone_ messages, used to "mark for deletion" all messages
//! of the given key.
//!
//! ## Example
//!
//! Here is how the code should be structured to handle the flow of messages coming out of
//! the [`__consumer_offsets`] topic.
//!
//! ```
//! use konsumer_offsets::{KonsumerOffsetsData, KonsumerOffsetsError};
//!
//! fn consume_consumer_offsets_message(k_bytes: Option<Vec<u8>>, p_bytes: Option<Vec<u8>>) {//!
//!     match KonsumerOffsetsData::try_from_bytes_vec(k_bytes, p_bytes) {
//!         Ok(kod) => {
//!             match kod {
//!                 KonsumerOffsetsData::OffsetCommit(offset_commit) => {
//!                     /* ... a consumer committed an offset for a partition in a topic ... */
//!                 }
//!                 KonsumerOffsetsData::GroupMetadata(group_metadata) => {
//!                     /* ... a consumer joined or leaved the group ... */
//!                 }
//!             }
//!         }
//!         Err(e) => {
//!             /* ... handle parsing error ... */
//!         }
//!     }
//! }
//! ```
//!
//! ## 1 topic, 2 different data types
//!
//! The messages (a.k.a. records) in [`__consumer_offsets`] are encoded using a _bespoke_ binary
//! protocol, and each message can contain 1 of 2 possible data types (at least as per Kafka 3):
//!
//! * [`OffsetCommit`]
//! * [`GroupMetadata`]
//!
//! Which data type is contained in a message is determined by parsing the first 2-bytes integer,
//! carrying a `message_version` code: this will be amongst the fields of both structs.
//!
//!
//! ### [`OffsetCommit`] a.k.a. "where is the consumer at?"
//!
//! Kafka uses code generation to materialise [`OffsetCommit`] into Java code,
//! and this is composed of 2 json definitions, that at compile time get turned into Java Classes:
//! [`OffsetCommitKey`] and [`OffsetCommitValue`].
//!
//! Assuming you are familiar with the [Consumer Group Protocol] (as discussed above): this
//! is what the Group Coordinator stores into the [`__consumer_offsets`] topic, when a consumer,
//! assignor of a specific _partition_, sends a request to the coordinator to _commit_ the _offset_
//! up to which it has consumed. It's _essentially_ informing the group coordinator that:
//!
//! > "I consumed this partition for this topic up to this offset".
//!
//! The information stored in [`OffsetCommit`] can be used to track
//! _who has consumed what until now_. Refer to its own documentation for further details.
//!
//! ### [`GroupMetadata`] a.k.a. "what are the members of this group?"
//!
//! Similarly to [`OffsetCommit`], Kafka uses code generation to materialise [`GroupMetadata`]
//! into Java code, and this is composed of 2 json definitions, that at compile time
//! get turned into Java Classes: [`GroupMetadataKey`] and [`GroupMetadataValue`].
//!
//! [`GroupMetadata`] contains the current state of a consumer group, and it used by the [Group
//! Coordinator] to track "which consumer is subscribed to what topic" and "which consumer
//! is assigned of which partition".
//!
//! Compared to [`OffsetCommit`], [`GroupMetadata`] appears _relatively infrequently_ in
//! [`__consumer_offsets`]: this is because it's usually produced when consumers join or leave
//! groups.
//!
//! ### [`KonsumerOffsetsData`] i.e. "making it rusty"
//!
//! The entry point to this crate is [`KonsumerOffsetsData`]: this is an enum where each variant
//! wraps one of the 2 possible data types seen above, as they are parsed from bytes coming
//! out of [`__consumer_offsets`].
//!
//! For convenience we provide 2 `try_from_*` factory methods:
//!
//! * [`KonsumerOffsetsData::try_from_bytes`]
//! * [`KonsumerOffsetsData::try_from_bytes_vec`]
//!
//! This is because some Kafka clients might return the `(key,payload)` _tuple_ as `[u8]`,
//! others as `Vec<u8>`.
//!
//! ## A few words about parsing Kafka _entrails_
//!
//! Kafka runs on the JVM, so it's limited to what the JVM supports.
//!
//! This crate deals with these limitations, and the data it parses and the fields we use to
//! store it, are a reflection of what Java supports. For example, in places where one would expect
//! an unsigned integer, like a string length or an ever increasing identifier, we use
//! the signed integers that Java supports.
//!
//! ### Java Primitive types in Rust
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
//! ### Bespoke formats used by `__consumer_offsets`
//!
//! In addition to fields of one of the primitive types above,
//! messages of this internal topic also contain a few more structured types:
//! _string_ and _vector of bytes_.
//!
//! #### Type: _string_
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
//! #### Type: _vector of bytes_
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
//! ## Disclaimer
//!
//! [Confluent] is a great place to start if you are looking for a commercial Kafka solution,
//! including PaaS, SaaS, training and more. **BUT** it is completely unrelated from this
//! crate and any reference to it is purely educational.
//!
//! [Primitive Types]: https://docs.oracle.com/javase/tutorial/java/nutsandbolts/datatypes.html
//! [`__consumer_offsets`]: https://kafka.apache.org/documentation/#impl_offsettracking
//! [Kafka]: https://kafka.apache.org/
//! [Consumer Group Protocol]: https://developer.confluent.io/learn-kafka/architecture/consumer-group-protocol/
//! [Confluent Developer]: https://developer.confluent.io/
//! [Confluent]: https://www.confluent.io/about/
//! [`OffsetCommitKey`]: https://github.com/apache/kafka/blob/trunk/core/src/main/resources/common/message/OffsetCommitKey.json
//! [`OffsetCommitValue`]: https://github.com/apache/kafka/blob/trunk/core/src/main/resources/common/message/OffsetCommitValue.json
//! [`GroupMetadataKey`]: https://github.com/apache/kafka/blob/trunk/core/src/main/resources/common/message/GroupMetadataKey.json
//! [`GroupMetadataValue`]: https://github.com/apache/kafka/blob/trunk/core/src/main/resources/common/message/GroupMetadataValue.json
//! [Log Compaction]: https://kafka.apache.org/documentation/#compaction
//! [Group Coordinator]: https://github.com/apache/kafka/blob/trunk/core/src/main/scala/kafka/coordinator/group/GroupCoordinator.scala
//! [Consumer Client]: https://github.com/apache/kafka/tree/trunk/clients/src/main/java/org/apache/kafka/clients/consumer
//! [`chrono::DateTime<Utc>`]: https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.from_utc
//! [`time::OffsetDateTime`]: https://time-rs.github.io/api/time/struct.OffsetDateTime.html#method.from_unix_timestamp_nanos
//! [serde]: https://crates.io/crates/serde
//!

mod errors;
mod group_metadata;
mod konsumer_offsets_data;
mod offset_commit;
mod utils;

pub use errors::*;
pub use group_metadata::*;
pub use konsumer_offsets_data::*;
pub use offset_commit::*;

#[cfg(any(
    all(feature = "ts_int", feature = "ts_chrono"),
    all(feature = "ts_int", feature = "ts_time"),
    all(feature = "ts_time", feature = "ts_chrono"),
))]
compile_error!("Features \"ts_int (default)\", \"ts_chrono\" and \"ts_time\" are mutually exclusive");
