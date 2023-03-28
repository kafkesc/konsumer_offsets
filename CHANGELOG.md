# v0.2.0 (20??-??-??)

## Breaking Changes

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

## Bug Fixes

## Notes

* Dependencies updates

# v0.1.1 (2023-03-13)

## Notes

* Dependencies updates


# v0.1.0 (2023-01-18)

## Features

* First release
* Complete parser for the content of the Kafka `__consumer_offsets` Topic

