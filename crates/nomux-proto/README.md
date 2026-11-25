# nomux-proto

The wire protocol of [nomux](https://github.com/nomux-sh): control messages and the terminal frame header.

The protocol is a draft. There is no public release.

## Contents

- `wire::v1`: the control messages (`Hello`, `Welcome`, `Envelope`, `Request`, `Response`, `Event`, `Error`). `prost` generates these types from the protobuf schemas.
- `frame`: the fixed 24-byte header of a terminal data frame, with encode and decode functions.
- `compat`: the control protocol version and the version rule of the handshake.

The generated code is a part of the crate. A build does not start `buf` or `protoc`.

The crate has no `unsafe` code.

## Example

```rust
use nomux_proto::compat::VersionRange;
use nomux_proto::wire::v1::{ClientKind, Hello};
use prost::Message;

let hello = Hello {
    version: VersionRange::CURRENT.version,
    oldest: VersionRange::CURRENT.oldest,
    capabilities: Vec::new(),
    client_kind: ClientKind::Cli.into(),
    token: String::new(),
};
let bytes = hello.encode_to_vec();
assert_eq!(Hello::decode(bytes.as_slice()), Ok(hello));
```

## Specification

The schemas and the specification documents are in the [proto repository](https://github.com/nomux-sh/proto).

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
