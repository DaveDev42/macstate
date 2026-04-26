use clap::Parser;
use serde_json::Value;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "macstate",
    version,
    about = "macOS system signals as JSON (network, power)"
)]
struct Cli {
    /// Show only the network subset.
    #[arg(long, conflicts_with_all = ["power", "query", "check"])]
    network: bool,

    /// Show only the power subset.
    #[arg(long, conflicts_with_all = ["network", "query", "check"])]
    power: bool,

    /// Print a single value at a dotted path (e.g. network.constrained).
    #[arg(short = 'q', long = "query", value_name = "PATH", conflicts_with = "check")]
    query: Option<String>,

    /// Exit 0 if the boolean at PATH is true, else 1. Useful in shell guards.
    #[arg(long, value_name = "PATH")]
    check: Option<String>,

    /// Print the JSON Schema describing the output and exit.
    #[arg(long, conflicts_with_all = ["network", "power", "query", "check"])]
    schema: bool,
}

const SCHEMA: &str = include_str!("../schema.json");

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.schema {
        // Re-parse and pretty-print so the output matches the rest of the CLI
        // (and so we fail loudly if the embedded file is ever malformed).
        let parsed: Value = serde_json::from_str(SCHEMA).expect("embedded schema is valid JSON");
        println!(
            "{}",
            serde_json::to_string_pretty(&parsed).expect("serialize schema")
        );
        return ExitCode::SUCCESS;
    }

    let state = macstate_core::State::collect();
    let full = serde_json::to_value(&state).expect("serialize state");

    if let Some(path) = cli.check.as_deref() {
        return match dig(&full, path) {
            Some(Value::Bool(true)) => ExitCode::SUCCESS,
            Some(Value::Bool(false)) => ExitCode::FAILURE,
            Some(other) => {
                eprintln!("macstate: --check expects a boolean at '{path}', got: {other}");
                ExitCode::from(2)
            }
            None => {
                eprintln!("macstate: no value at path '{path}'");
                ExitCode::from(2)
            }
        };
    }

    if let Some(path) = cli.query.as_deref() {
        match dig(&full, path) {
            Some(v) => {
                match v {
                    Value::String(s) => println!("{s}"),
                    other => println!("{other}"),
                }
                return ExitCode::SUCCESS;
            }
            None => {
                eprintln!("macstate: no value at path '{path}'");
                return ExitCode::from(2);
            }
        }
    }

    let out = if cli.network {
        serde_json::to_string_pretty(&state.network)
    } else if cli.power {
        serde_json::to_string_pretty(&state.power)
    } else {
        serde_json::to_string_pretty(&state)
    }
    .expect("serialize output");

    println!("{out}");
    ExitCode::SUCCESS
}

fn dig<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(root, |acc, key| acc.get(key))
}
