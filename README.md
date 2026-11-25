# proto

The nomux wire protocol: protobuf schemas for control messages, the terminal frame specification, the holder protocol specification, and the capability flag registry.

This repository is a part of [nomux](https://github.com/nomux-sh), a native and lightweight agentic development environment. Sessions stay alive on the host when the client disconnects.

## Status

In development. There is no public release.

The protocol is a draft until ADR-0002 (protocol: protobuf and binary frame) is accepted. This repository contains the structure, the tooling, and the handshake. Each later milestone adds the messages of its services.

## Layout

```
proto/nomux/wire/v1/         protobuf schemas, package nomux.wire.v1
  handshake.proto            Hello, Welcome, ClientKind
  envelope.proto             Envelope, Request, Response, Event, Error
spec/
  frame.md                   terminal frame header (binary, not protobuf)
  holder.md                  holder protocol: versioning rule, open questions
  capabilities.md            capability flag registry
  compatibility.md           version rule of the handshake, exit code 5
crates/nomux-proto/          Rust crate: generated prost types, frame header codec
  src/generated/             output of `buf generate` (committed)
tools/protoc-gen-nomux-prost/  local buf plugin that runs prost-build
scripts/check-generated.sh   fails if the generated code is stale
buf.yaml, buf.gen.yaml       buf configuration
deny.toml                    cargo-deny configuration
```

## Tools

| Tool | Version | Used for |
|------|---------|----------|
| Rust | 1.100.0 (pinned in `rust-toolchain.toml`) | build, test |
| [buf](https://github.com/bufbuild/buf/releases) | 1.75.0 | lint, format, breaking change check, code generation |
| [cargo-nextest](https://nexte.st) | 0.9.152 | tests |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | 0.22.1 | license and advisory check |

`protoc` is not necessary. `buf` compiles the schemas.

To build the crate `nomux-proto` or a crate that depends on it, only Rust is necessary. `buf` is necessary only to change the schemas.

## Code generation

The generated Rust code is committed in `crates/nomux-proto/src/generated/`. The crate has no `build.rs`.

`buf generate` starts a plugin that is a binary in this workspace (`tools/protoc-gen-nomux-prost`). The plugin calls `prost-build` with the descriptors from `buf`. `Cargo.lock` pins `prost-build` and the `prost` runtime to the same version. No remote plugin runs and no schema registry is used.

After you change a `.proto` file, do these steps:

1. Format the schemas:

   ```sh
   buf format -w
   ```

2. Generate the code:

   ```sh
   buf generate
   ```

3. Commit the schemas and the generated code in the same commit.

CI runs `scripts/check-generated.sh`. The check fails if the committed code is not the output of `buf generate`.

To generate code for a different language (for example TypeScript, Go, or Python), add a plugin to a local copy of `buf.gen.yaml`. This repository commits only the Rust code.

## Checks

Run these commands from the repository root before you open a pull request:

```sh
buf lint
buf format --diff --exit-code
buf build
scripts/check-generated.sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
```

CI runs the same checks with the reusable workflows of the [`nomux-sh/.github`](https://github.com/nomux-sh/.github) repository.

`buf breaking` does not run yet, because no tag exists. After the first tag, CI compares each pull request with the last tag.

A fuzz target for the frame codec is not in this repository yet. Milestone M1 adds it.

## Versioning rules

- **Field numbers are permanent.** Do not change the number or the type of a field. Do not use the number of a deleted field again: add a `reserved` statement for the number and the name when you delete a field.
- **Field number plan.** In the `body` and `result` oneofs of the envelope, each service has its own block of 100 field numbers. The plan is at the top of `proto/nomux/wire/v1/envelope.proto`.
- **Free numbers are not `reserved`.** `buf breaking` does not let you delete a reserved range. Use `reserved` only for numbers that must never be used again.
- **Breaking changes.** `buf breaking` uses the `FILE` rule set. A change that fails this check needs a new package (`nomux.wire.v2`).
- **Control protocol version.** An integer, plus capability flags. See [spec/compatibility.md](spec/compatibility.md) and [spec/capabilities.md](spec/capabilities.md).
- **Holder protocol.** The daemon supports all holder versions of the same major product version. See [spec/holder.md](spec/holder.md).
- **Crate version.** The crate `nomux-proto` uses SemVer. The minimum supported Rust version is not set yet. Only the pinned toolchain is tested.
- **Commits.** Use Conventional Commits and sign off each commit (`git commit -s`).

## Security

To report a vulnerability, obey the [security policy](https://github.com/nomux-sh/.github/blob/main/SECURITY.md) of the nomux-sh organization.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

The specification documents in `spec/` are licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

Copyright The nomux Authors.
