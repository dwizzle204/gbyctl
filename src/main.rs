//! Binary entrypoint.

use anyhow::Result;
use clap::Parser;

use gbyctl::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse_from(normalize_args(std::env::args()));
    gbyctl::cli::dispatch(cli)
}

fn normalize_args<I>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut args: Vec<String> = args.into_iter().collect();
    if args.len() <= 2 {
        return args;
    }

    let program = args.remove(0);
    let mut flags = Vec::new();
    let mut words = Vec::new();
    let mut first_non_flag: Option<&str> = None;

    for arg in &args {
        if first_non_flag.is_none() && !arg.starts_with('-') {
            first_non_flag = Some(arg);
        }
    }

    if let Some(token) = first_non_flag
        && is_explicit_command(token)
    {
        let mut passthrough = Vec::with_capacity(args.len() + 1);
        passthrough.push(program);
        passthrough.extend(args);
        return passthrough;
    }

    for arg in args {
        if arg.starts_with('-') {
            flags.push(arg);
        } else {
            words.push(arg);
        }
    }

    if words.is_empty() {
        let mut passthrough = Vec::with_capacity(flags.len() + 1);
        passthrough.push(program);
        passthrough.extend(flags);
        return passthrough;
    }

    let mut normalized = Vec::with_capacity(flags.len() + 2);
    normalized.push(program);
    normalized.extend(flags);
    normalized.push(words.join(" "));
    normalized
}

fn is_explicit_command(token: &str) -> bool {
    matches!(
        token,
        "setup"
            | "doctor"
            | "inspect-storage"
            | "service-status"
            | "package-status"
            | "install"
            | "troubleshoot-firewall"
            | "diagnose"
            | "logs"
            | "maintenance"
            | "resize-root"
            | "help"
            | "--help"
            | "-h"
            | "--version"
            | "-V"
    )
}
