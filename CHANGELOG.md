# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

[git-cliff](https://git-cliff.org) writes this file from the commit subjects.
To see the changes that are not in a version, run `git cliff --unreleased`.

## [0.7.0] - 2026-11-26

### Added

- Add the Agent service and the Events service
## [0.6.0] - 2026-11-26

### Added

- Add the request for the memory of each session
## [0.5.0] - 2026-11-25

### Added

- Add the HistoryGap and Restored frames and their capability flag
- Add restore, capture and the states of a session with no process

### Documentation

- List the decided questions of the holder protocol
## [0.4.0] - 2026-11-25

### Added

- Add the Ext service for the extensions of a host
## [0.3.0] - 2026-11-25

### Added

- Add the attach, detach, input and resize messages
- Add the End frame and the epoch of this version

### Documentation

- Point the holder protocol outline to its specification
- Specify the terminal data path of an attach
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
