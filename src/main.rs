use cargo_metadata::PackageId;
use colored::*;
use itertools::Itertools;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

#[macro_use]
mod macros;

const HELP_MSG: &str = r#"cargo-avrdude
Builds the binary and passes it to an arbitrary AVRDUDE command

USAGE:
    cargo avrdude [FLAGS] [<cargo_options>...]

FLAGS:
    -h, --help    Prints help information

ARGS:
    <cargo_options>...    Options passed to cargo
"#;

struct BinaryInfo {
    name: String,
    package_id: PackageId,
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CargoAvrdudeMetadata {
    args: Vec<String>,
}

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("{}", HELP_MSG);

        exit(0);
    }

    match args.first() {
        Some(arg) if arg == "avrdude" => {
            args.remove(0);
        }
        _ => {
            warning!("expected `avrdude` as the second argument. cargo-avrdude is intended to be invoked as a cargo subommand");
        }
    };

    let output = Command::new("cargo")
        .arg("build")
        .args(args)
        .arg("--message-format=json")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|error| exit_with_error!("{}", error))
        .wait_with_output()
        .unwrap_or_else(|error| exit_with_error!("{}", error));

    if !output.status.success() {
        exit(1);
    }

    let reader = BufReader::new(output.stdout.as_slice());
    let binary_infos = cargo_metadata::Message::parse_stream(reader)
        .filter_map(|message_res| {
            let message = message_res.ok()?;

            let artifact = if let cargo_metadata::Message::CompilerArtifact(artifact) = message {
                artifact
            } else {
                return None;
            };

            let name = artifact.target.name.clone();
            let package_id = artifact.package_id.clone();
            let path = artifact.executable?.into();

            Some(BinaryInfo {
                name,
                package_id,
                path,
            })
        })
        .collect::<Vec<_>>();

    let BinaryInfo {
        package_id: binary_package_id,
        path: binary_path,
        ..
    } = match binary_infos.len() {
        0 => {
            warning!("cargo did not generate a binary. Nothing else to do");

            exit(0);
        },
        1 => binary_infos.get(0).unwrap(), // Unwrap-safety: one element is present in the Vec
        _ => exit_with_error!("cargo generated more than one binary. Please specify one with `--bin <name>`. Generated binaries: {}", binary_infos.iter().map(|info| &info.name).join(", ")),
    };

    let metadata = cargo_metadata::MetadataCommand::new()
        .exec()
        .unwrap_or_else(|error| exit_with_error!("{}", error));

    let package = metadata
        .packages
        .iter()
        .find(|package| &package.id == binary_package_id)
        .unwrap_or_else(|| {
            exit_with_error!(
                "couldn't find package {} in cargo metadata",
                binary_package_id
            )
        });

    let CargoAvrdudeMetadata { args } = if let Some(value) = package.metadata.get("cargo_avrdude") {
        serde_json::from_value::<CargoAvrdudeMetadata>(value.to_owned()).unwrap_or_else(|error| {
            exit_with_error!("invalid `package.metadata.cargo_avrdude` structure: {}", error)
        })
    } else {
        exit_with_error!(
            "please provide a `package.metadata.cargo_avrdude.args` in {}",
            package.manifest_path
        );
    };

    let args = args
        .into_iter()
        .map(|arg| arg.replace("{}", &binary_path.to_string_lossy()))
        .collect::<Vec<_>>();

    progress!("Flashing", "{}", binary_path.display());

    let output = Command::new("avrdude")
        .args(args)
        .output()
        .unwrap_or_else(|error| exit_with_error!("{}", error));

    if output.status.success() {
        progress!("Flashed", "{}", binary_path.display());
    } else {
        let mut printed_error = false;

        for line_res in output.stderr.as_slice().lines() {
            let line = line_res.unwrap_or_else(|error| exit_with_error!("{}", error));

            if line.starts_with("avrdude:") {
                if line[9..].starts_with("error:") {
                    error!("{}", &line[16..]);

                    printed_error = true;
                }
            }
        }

        if !printed_error {
            error!("unknown error");
        }

        exit(1);
    }
}
