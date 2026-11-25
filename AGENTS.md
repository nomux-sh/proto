# AGENTS.md

Rules for coding agents and for people who work in this repository. `CLAUDE.md` and `QWEN.md` refer to this file.

## This repository

`nomux-sh/proto` is the nomux wire protocol: the protobuf schemas of the control messages, the specification documents, and the Rust crate `nomux-proto` that crates.io publishes.

| Path | Responsibility |
|---|---|
| `proto/nomux/wire/v1/` | Protobuf schemas of the package `nomux.wire.v1`: envelope, handshake, Host service and Session service. |
| `spec/` | Specification documents: control channel, terminal frame header, holder protocol, capability flag registry, version rule. |
| `crates/nomux-proto/` | The published crate: generated control messages, terminal frame header codec, version rule. |
| `crates/nomux-proto/src/generated/` | Output of `buf generate`. Committed. Not edited by hand. |
| `crates/nomux-proto/tests/` | Tests of the protobuf wire format. |
| `tools/protoc-gen-nomux-prost/` | The `buf` plugin that writes the generated code. Not published. |
| `scripts/check-generated.sh` | Fails when the generated code is stale. |
| `buf.yaml`, `buf.gen.yaml` | `buf` module, lint rules, breaking change rules, code generation. |
| `Cargo.toml`, `deny.toml`, `rust-toolchain.toml` | Workspace members and shared versions, license and source policy, pinned toolchain. |
| `cliff.toml`, `CHANGELOG.md` | Changelog configuration and the changelog. |
| `.github/workflows/` | `ci.yml` (checks), `pull-request.yml` (sign-off, commit messages), `release.yml` (publishes a version tag). |

## Rules for this repository

- A field number and an enum value are permanent. Do not change the number or the type of a field. When you delete a field, add a `reserved` statement for its number and its name.
- A change that fails `buf breaking` needs a new package (`nomux.wire.v2`). Do not change the rule set to make the check pass.
- When you change a `.proto` file, run `buf format -w` and `buf generate`. Commit the schema and the generated code in the same commit.
- Do not edit a file in `crates/nomux-proto/src/generated/`.
- The crate `nomux-proto` builds with Rust only. Do not add a `build.rs`, and do not add a step that needs `buf` or `protoc` to a build of the crate.
- `prost`, `prost-build` and `prost-types` have the same version in `Cargo.toml`. Change them together and generate the code again.
- `spec/frame.md`, `crates/nomux-proto/src/frame.rs` and the layout tests in that file describe one layout. Change them together.
- `spec/compatibility.md` and `crates/nomux-proto/src/compatibility.rs` describe one version rule. Change them together.
- Register a capability flag in `spec/capabilities.md` before a schema or a frame uses it.
- A test in this repository guards the wire format or the version rule. A test that only encodes and decodes a message with `prost` does not.
- Pin each action in a workflow to a full commit SHA with a version comment.
- Do not publish the crate from a computer. The release workflow publishes it from a version tag. The steps are in the [release guide](https://github.com/nomux-sh/.github/blob/main/RELEASING.md).

## Checks

The tools and their versions are in [README.md](README.md#tools). Run these commands from the repository root before a push. CI runs the same commands.

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

If you changed `AGENTS.md`, also run this command with a clone of the [`.github`](https://github.com/nomux-sh/.github) repository next to this repository:

```sh
sh ../.github/standards/sync.sh . && git diff --exit-code
```

<!-- nomux:common:start -->
## Organization rules

These rules apply to each repository of the nomux-sh organization. The source of this block is `standards/AGENTS.common.md` in the [`.github`](https://github.com/nomux-sh/.github) repository. Do not edit the block in other repositories. CI compares it with the source.

### Before you change something

- Read the code around the change. Use the names, the structure and the style that the repository already has.
- Use the terms of the [glossary](https://github.com/nomux-sh/docs/blob/main/src/content/docs/reference/glossary.md). One concept has one name in code, CLI, UI and documentation.
- An architecture change needs an ADR in the [`docs`](https://github.com/nomux-sh/docs) repository before the implementation.
- Check versions, APIs and file paths before you use them. Do not guess.

### Scope

- Make only the change that the task asks for. Do not add options, abstractions or helpers that the task does not need.
- Do not add a placeholder that looks like an implementation. If work is deferred, say it in the pull request.
- Validate input at system boundaries. Trust internal code.

### Code

- Keep it simple (KISS). Build only what the task needs now (YAGNI). Have one source for each piece of logic (DRY).
- Write modern, readable code. Do not write clever code that the reader must decode.
- One type, one module and one file have one responsibility. Do not make a type that knows or does everything. Split it before it grows.
- Keep the file tree tidy. A new file goes where its neighbours have the same responsibility. Do not make catch-all directories or files such as `utils`, `common`, `helpers` or `misc`.
- Give each file, type and function a name that says what it is or does. Do not use abbreviations that the glossary does not have.
- Keep functions short and at one level of abstraction. Do not nest deeply. Return early.
- Do not duplicate code. If the same logic is necessary in a second place, move it to one place and call it.
- Comments: none by default. Write one line only for a reason that the code cannot show.
- Do not swallow errors. Do not add retries without a known cause.
- The performance targets of the project come before features. A change that breaks a target is not merged.
- Do not copy code from GPL, AGPL or Commons Clause projects.

### Tests

- Complete the code first. Write the tests last, and run them when the work is complete.
- Write the tests that matter: edge cases, behaviour that can break later, and the proof that the current behaviour is correct.
- Do not write a test only to have a test. Do not test what is obvious or what the compiler already guarantees.
- A test must fail when the behaviour that it guards is removed or moved.
- Do not mock the thing that the test examines.

### CI

- CI must be fast. Reviews and CI are the only things that slow a merge.
- Cache dependencies and build output. Run independent jobs in parallel. Cancel a run when a newer commit arrives.
- Put slow work (long fuzz runs, benchmarks, full matrix builds) in scheduled runs, not in pull request runs.
- When you add a CI step, measure the time that it adds and say it in the pull request.

### Text

- Write code, comments, documentation, commit messages and pull requests in English.
- Use short sentences, the active voice and one word for one concept. Put the condition before the instruction.
- Do not use these words: delve, robust, seamless, seamlessly, leverage, utilize, harness, pivotal, intricate, furthermore, moreover, notably, cutting-edge, best-in-class.

### Commits

- One subject line: `type: description`. No scope. 72 characters at most.
- Types: `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`. Add `!` before the colon for a breaking change.
- The description says what changed, in plain English. Do not name a phase, a milestone or a ticket.
- Sign off each commit with `git commit -s`. `Signed-off-by` is the only trailer.
- Do not add `Co-authored-by`, a tool name or a "generated with" line to a commit or to a pull request.
- Enable the commit hook one time in each clone: `git config core.hooksPath .githooks`.

### Pull requests

- One logical change in each pull request. The branch name is short and names the change.
- The title has the format of a commit subject, 70 characters at most.
- The description says what changed and why, then shows the commands that you ran and their real result.
- Each commit in the pull request obeys the commit rules. The maintainers merge with a rebase, thus each commit goes to `main` unchanged.
- Do not merge when a check fails. Do not disable a check to make it pass.

### Never commit

- Secrets, tokens, keys or passwords.
- Paths, host names or account names of a local machine.
- Generated build output, editor files or local configuration.

### Before you push

1. Run the checks that the "Checks" section of this file lists.
2. Read your diff from top to bottom.
3. Make sure that the changelog and the documentation agree with the change.
<!-- nomux:common:end -->
