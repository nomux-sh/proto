# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

[git-cliff](https://git-cliff.org) writes this file from the commit subjects.
To see the changes that are not in a version, run `git cliff --unreleased`.

## [0.2.0] - 2026-11-25

### Added

- Add the Host and Session control messages

### Documentation

- Specify the control channel and its services
## [0.1.0] - 2026-11-25

### Added

- Add buf setup, handshake and envelope schemas
- Add generated types and frame header codec

### Changed

- Rename compat to compatibility and remove FrameKind::ALL

### Documentation

- Add frame, holder, capability and compatibility specs
- Describe layout, code generation, checks and versioning
- Say what the repository has now and link the roadmap
