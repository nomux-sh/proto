# nomux-proto

The wire protocol of [nomux](https://github.com/nomux-sh): control messages and the terminal frame header.

## Status

In development. Versions earlier than 1.0.0 are pre-releases: interfaces can change between minor versions.

The crate has these parts now:

- `wire::v1`: the control messages. These are the envelope messages (`Envelope`, `Request`, `Response`, `Event`, `Error`), the handshake messages (`Hello`, `Welcome`), the Host service (`Info`, `Stop`), the Session service (`Create`, `List`, `Kill`, `Attach`, `Detach`, `Input`, `Resize`, `Start`, `Capture`, `SetRestorePolicy`, `Memory`), the Ext service (`List`, `Inspect`, `Install`, `Remove`, `SetEnabled`, `Call`), the Agent service (`Tree`, `Explain`, `SetStatus`), the Events service (`Subscribe`), the Hooks service (`Event`, `Plan`, `Adapters`) and the payloads of the adapter extensions. `prost` generates these types from the protobuf schemas.
- `frame`: the fixed 24-byte header of a terminal data frame and the payload of the `End` frame, with encode and decode functions.
- `compatibility`: the control protocol version and the version rule of the handshake.
- `capabilities`: the names of the capability flags that are `active`.

The layout of the frame header is a draft. The screen diff, the acknowledgement, the credit window and the messages of the other services are future work. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

The generated code is a part of the crate. A build does not start `buf` or `protoc`.

The crate has no `unsafe` code. The minimum supported Rust version is 1.85.

## Example

```rust
use nomux_proto::compatibility::{VersionRange, negotiate};
use nomux_proto::frame::{FrameHeader, FrameKind};
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

let daemon = VersionRange { version: 1, oldest: 1 };
assert_eq!(negotiate(VersionRange::CURRENT, daemon), Ok(1));

let header = FrameHeader {
    kind: FrameKind::Output,
    session: 1,
    epoch: 1,
    seq: 0,
    length: 5,
};
let bytes = header.encode().expect("the length is not larger than the limit");
assert_eq!(FrameHeader::decode(&bytes), Ok(header));
```

## Specification

The schemas and the specification documents are in the [proto repository](https://github.com/nomux-sh/proto).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
