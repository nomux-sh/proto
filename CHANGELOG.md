# Changelog

This file lists the important changes to this repository.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The crate `nomux-proto` uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `buf` configuration (`buf.yaml`, `buf.gen.yaml`) for the package `nomux.wire.v1`.
- Handshake schema: `Hello`, `Welcome`, and `ClientKind`.
- Envelope schema: `Envelope`, `Request`, `Response`, `Event`, `Error`, and `ErrorCode`.
- Crate `nomux-proto` with the generated `prost` types, the terminal frame header codec, and the version rule of the handshake.
- Local `buf` plugin `protoc-gen-nomux-prost` and the script `scripts/check-generated.sh`.
- Specification documents: `spec/frame.md`, `spec/holder.md`, `spec/capabilities.md`, and `spec/compatibility.md`.
- CI workflow that calls the reusable workflows of the `nomux-sh/.github` repository.
