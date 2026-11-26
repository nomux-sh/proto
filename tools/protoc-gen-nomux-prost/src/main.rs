//! Local `buf` plugin that generates the `prost` code of `nomux-proto`.
//!
//! `buf generate` starts this binary (see `buf.gen.yaml`). The binary reads a
//! `CodeGeneratorRequest` from standard input and writes a
//! `CodeGeneratorResponse` to standard output. `prost-build` generates the
//! code from the descriptors in the request, thus `protoc` is not necessary.
//!
//! Cargo.lock pins `prost-build`. The generator version and the `prost`
//! runtime version of `nomux-proto` change together.

use std::io::{self, Read, Write};

use prost::Message;
use prost_types::FileDescriptorSet;
use prost_types::compiler::code_generator_response::{Feature, File};
use prost_types::compiler::{CodeGeneratorRequest, CodeGeneratorResponse};

fn main() -> io::Result<()> {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    let request = CodeGeneratorRequest::decode(input.as_slice())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    // A plugin reports a generation failure in the response, not with a
    // non-zero exit status.
    let response = match generate(request) {
        Ok(file) => CodeGeneratorResponse {
            file,
            supported_features: Some(Feature::Proto3Optional as u64),
            ..Default::default()
        },
        Err(error) => CodeGeneratorResponse {
            error: Some(error.to_string()),
            ..Default::default()
        },
    };

    io::stdout().write_all(&response.encode_to_vec())
}

/// Generates one Rust file for each protobuf package in the request.
fn generate(request: CodeGeneratorRequest) -> io::Result<Vec<File>> {
    let out_dir = tempfile::tempdir()?;
    let descriptors = FileDescriptorSet {
        file: request.proto_file,
    };
    prost_build::Config::new()
        .out_dir(out_dir.path())
        // `Hello` contains the device token, and `AgentSetStatusRequest` and
        // `HookEventRequest` contain the token of a Session. `nomux-proto`
        // implements `Debug` for them by hand and does not print the token.
        .skip_debug([
            ".nomux.wire.v1.Hello",
            ".nomux.wire.v1.AgentSetStatusRequest",
            ".nomux.wire.v1.HookEventRequest",
        ])
        // A message with an Agent run or with a status is much larger than
        // the other messages of its oneof.
        .boxed(".nomux.wire.v1.Event.body.agent_run_changed")
        .boxed(".nomux.wire.v1.Response.result.agent_tree")
        .boxed(".nomux.wire.v1.Response.result.agent_explain")
        .compile_fds(descriptors)?;

    let mut files = Vec::new();
    for entry in std::fs::read_dir(out_dir.path())? {
        let entry = entry?;
        let name = entry.file_name().into_string().map_err(|name| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("file name is not UTF-8: {name:?}"),
            )
        })?;
        files.push(File {
            name: Some(name),
            content: Some(std::fs::read_to_string(entry.path())?),
            ..Default::default()
        });
    }
    // The order of `read_dir` is not stable. Sort to get the same response
    // for the same request.
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}
