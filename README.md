# proto

The nomux wire protocol: protobuf schemas for control messages, the terminal frame specification, the holder protocol specification, and the capability flag registry.

This repository is a part of [nomux](https://github.com/nomux-sh), a native and lightweight agentic development environment. Sessions stay alive on the host when the client disconnects.

## Status

In development. Versions earlier than 1.0.0 are pre-releases: interfaces can change between minor versions.

The repository has these parts now:

- The handshake messages (`Hello`, `Welcome`) and the envelope messages (`Envelope`, `Request`, `Response`, `Event`, `Error`) of the control protocol.
- The Host service (`Info`, `Stop`) and the Session service (`Create`, `List`, `Kill`, `Attach`, `Detach`, `Input`, `Resize`) of the control protocol.
- The codec for the header of a terminal data frame and for the payload of the `End` frame.
- The version rule: how a client and a daemon select a control protocol version.

The layout of the frame header is a draft. [spec/frame.md](spec/frame.md) specifies the terminal data path of an attach and lists the open items. The screen diff, the acknowledgement, the credit window and the messages of the other services are future work. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

[ADR-0002](https://github.com/nomux-sh/docs/blob/main/adr/0002-protocol-protobuf-and-binary-frames.md) records the decision to use protobuf for control messages and binary frames for terminal data.

The crate [`nomux-proto`](https://crates.io/crates/nomux-proto) is the published form of this repository. Its documentation is on [docs.rs](https://docs.rs/nomux-proto).

## Layout

```
proto/nomux/wire/v1/         protobuf schemas, package nomux.wire.v1
  handshake.proto            Hello, Welcome, ClientKind
  envelope.proto             Envelope, Request, Response, Event, Error
  host.proto                 Host service: Info, Stop
  session.proto              Session service: Create, List, Kill, Attach, Detach, Input, Resize
spec/
  control.md                 control channel: framing, error codes, services and their messages
  frame.md                   terminal frames (binary, not protobuf): header, payloads, terminal data path
  holder.md                  holder protocol: versioning rule, open questions
  capabilities.md            capability flag registry
  compatibility.md           version rule of the handshake, exit code 5
crates/nomux-proto/          Rust crate: generated prost types, frame header codec, version rule
  src/generated/             output of `buf generate` (committed)
tools/protoc-gen-nomux-prost/  local buf plugin that runs prost-build
scripts/check-generated.sh   fails if the generated code is stale
buf.yaml, buf.gen.yaml       buf configuration
deny.toml                    cargo-deny configuration
cliff.toml, CHANGELOG.md     git-cliff configuration, changelog
AGENTS.md                    rules for contributors and coding agents
```

## Tools

| Tool | Version | Used for |
|------|---------|----------|
| Rust | 1.100.0 (pinned in `rust-toolchain.toml`) | build, test |
| [buf](https://github.com/bufbuild/buf/releases) | 1.75.0 | lint, format, breaking change check, code generation |
| [cargo-nextest](https://nexte.st) | 0.9.152 | tests |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | 0.22.1 | license and advisory check |
| [cargo-machete](https://github.com/bnjbvr/cargo-machete) | 0.10.1 | unused dependency check |
| [git-cliff](https://git-cliff.org) | 2.14.2 | changelog |
| [actionlint](https://github.com/rhysd/actionlint) | 1.7.12 | workflow lint |

`protoc` is not necessary. `buf` compiles the schemas.

To build the crate `nomux-proto` or a crate that depends on it, only Rust is necessary. `buf` is necessary only to change the schemas.

## Code generation

The generated Rust code is committed in `crates/nomux-proto/src/generated/`. The crate has no `build.rs`.

`buf generate` starts a plugin that is a binary in this workspace (`tools/protoc-gen-nomux-prost`), and the plugin calls `prost-build` with the descriptors from `buf`. `Cargo.lock` pins `prost-build` and the `prost` runtime to the same version, and no remote plugin or schema registry is used. A `build.rs` is not used, because it makes each build of the crate need `protoc`.

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
cargo test --workspace --doc
cargo deny check
cargo machete
cargo package -p nomux-proto --locked
actionlint
```

CI runs the same checks with the reusable workflows of the [`nomux-sh/.github`](https://github.com/nomux-sh/.github) repository. `cargo package` builds the crate from the packaged files only. Thus the check fails if the package needs a file that it does not have.

### Breaking change check

The `last-tag` job of `.github/workflows/ci.yml` finds the newest tag with the format `vX.Y.Z` and gives `.git#tag=vX.Y.Z` to the `breaking-against` input of `buf-ci.yml`.

- If no tag exists, the input is empty and `buf breaking` does not run.
- After the first tag (`v0.1.0`) is pushed, each subsequent CI run compares the schemas with the newest tag. A change to the workflow is not necessary.

To run the same check on your computer:

```sh
buf breaking --against '.git#tag=v0.1.0'
```

A fuzz target for the frame codec is not in this repository. The [roadmap](https://github.com/nomux-sh/docs/blob/main/src/content/docs/project/roadmap.md) gives the plan.

## Releases

The [release guide](https://github.com/nomux-sh/.github/blob/main/RELEASING.md) of the organization gives the steps. In this repository:

- `git-cliff` writes `CHANGELOG.md` from the commit subjects. The configuration is `cliff.toml`.
- A push of a tag `vX.Y.Z` starts `.github/workflows/release.yml`. The workflow makes sure that the tag is equal to the crate version, that the commit is on `main` and that `CHANGELOG.md` has a section for the version. Then it runs the tests, publishes `nomux-proto` to crates.io and makes the GitHub release from the changelog section.
- The workflow needs the organization secret `CARGO_REGISTRY_TOKEN`.

## Versioning rules

- **Field numbers are permanent.** Do not change the number or the type of a field. Do not use the number of a deleted field again: add a `reserved` statement for the number and the name when you delete a field.
- **Field number plan.** In the `body` and `result` oneofs of the envelope, each service has its own block of 100 field numbers. The plan is at the top of `proto/nomux/wire/v1/envelope.proto`.
- **Free numbers are not `reserved`.** `buf breaking` does not let you delete a reserved range. Use `reserved` only for numbers that must never be used again.
- **Breaking changes.** `buf breaking` uses the `FILE` rule set. A change that fails this check needs a new package (`nomux.wire.v2`).
- **Control protocol version.** An integer, plus capability flags. See [spec/compatibility.md](spec/compatibility.md) and [spec/capabilities.md](spec/capabilities.md).
- **Holder protocol.** The daemon supports all holder versions of the same major product version. See [spec/holder.md](spec/holder.md).
- **Crate version.** The crate `nomux-proto` uses SemVer. The minimum supported Rust version is 1.85 (`rust-version` in the crate manifest). CI builds the crate with that version.
- **Commits.** The commit rules are in [AGENTS.md](AGENTS.md).

## Security

To report a vulnerability, obey the [security policy](https://github.com/nomux-sh/.github/blob/main/SECURITY.md) of the nomux-sh organization.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

The specification documents in `spec/` are licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

Copyright The nomux Authors.
