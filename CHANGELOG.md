# v0.3.1 (2024-05-20)

## Notes

* Deps upgrade

# v0.3.0 (2023-06-30)

## Features

* Adding `try_from` implementations for `&[u8]` and `Vec<u8>` input for
  `ConsumerProtocolAssignment` and `ConsumerProtocolSubscription` types.

## Bug Fixes

* Added missing documentation for `MemberMetadata.group_instance_id` field.

# v0.2.2 (2023-06-23)

## Bug Fixes

* Fixed visibility of fields for `subscription: ConsumerProtocolSubscription` and
  `assignment: ConsumerProtocolAssignment`, used by the `MemberMetadata` type.
  Their fields are now `pub`lic.

# v0.2.1 (2023-06-18)

## Notes

* Dependencies updates.

# v0.2.0 (2023-03-28)

## Features

* `ts_int`: Default feature flag to represent timestamps using `i64` (same as previous version)
* `ts_chrono`: Optional feature flag to represent timestamps using
  [`chrono::DateTime<Utc>`](https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.from_utc)
  ([I#2](https://github.com/kafkesc/konsumer_offsets/issues/2))
* `ts_time`: Optional feature flag to represent timestamps using
  [`time::OffsetDateTime`](https://time-rs.github.io/api/time/struct.OffsetDateTime.html#method.from_unix_timestamp_nanos)
  ([I#2](https://github.com/kafkesc/konsumer_offsets/issues/2))
* `serde`: Optional feature flag to enable [serde](https://crates.io/crates/serde)-based
  serialization/deserialization, for all the types exported by the crate
  ([I#3](https://github.com/kafkesc/konsumer_offsets/issues/3))

## Enhancements

* Enforced marker traits `Sized + Send + Sync + Unpin` for all exported
  types ([I#1](https://github.com/kafkesc/konsumer_offsets/issues/1))
* Enabled use of the 
  ["sparse"](https://blog.rust-lang.org/inside-rust/2023/01/30/cargo-sparse-protocol.html)
  protocol when interacting with the https://crates.io registry

## Notes

* Dependencies updates.

# v0.1.1 (2023-03-13)

## Notes

* Dependencies updates.


# v0.1.0 (2023-01-18)

## Features

* First release
* Complete parser for the content of the Kafka `__consumer_offsets` Topic.

