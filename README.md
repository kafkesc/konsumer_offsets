# konsumer_offsets [![CI_s]][CI_l] [![Doc_s]][Doc_l] [![Ver_s]][Ver_l] [![Down_s]][Down_l] ![Lic_s]

[CI_s]: https://img.shields.io/github/actions/workflow/status/kafkesc/konsumer_offsets/ci.yml?branch=main&label=CI&logo=Github&style=flat-square
[CI_l]: https://github.com/kafkesc/konsumer_offsets/actions/workflows/ci.yml
[Down_s]: https://img.shields.io/crates/d/konsumer_offsets?logo=rust&style=flat-square&label=DOWN
[Down_l]: https://crates.io/crates/konsumer_offsets
[Ver_s]: https://img.shields.io/crates/v/konsumer_offsets?logo=rust&style=flat-square&label=VER
[Ver_l]: https://crates.io/crates/konsumer_offsets/versions
[Doc_s]: https://img.shields.io/docsrs/konsumer_offsets?logo=rust&style=flat-square&label=DOC
[Doc_l]: https://docs.rs/konsumer_offsets/latest/konsumer_offsets/
[Lic_s]: https://img.shields.io/crates/l/konsumer_offsets?style=flat-square&label=L

**A library crate to parse the content of the [Kafka] [`__consumer_offsets`] internal topic.**

---

## Features

* **Most complete parser for [`__consumer_offsets`] messages out there**
* Reverse-engineering of Kafka 3.x parsing logic, making it retro-compatible by default
* Able to parse the _subscription_ and _assignment_ data contained in `GroupMetadata` messages:
  beyond what even the Kafka own parser can do
* Every struct and field is well documented
* Internal parsing functions are also documented and have references to the code they are based upon:
  if you read the code, you can go correlate to the [Kafka] codebase that its imitating
* Parsing is based on [bytes_parser] and errors on [thiserror], so it's easy to read
  and handles result errors idiomatically

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[Kafka]: https://kafka.apache.org/
[`__consumer_offsets`]: https://kafka.apache.org/documentation/#impl_offsettracking
[bytes_parser]: https://crates.io/crates/bytes_parser
[thiserror]: https://crates.io/crates/thiserror