use crate::clap::cli;
use crate::commands::build::build_command;
use crate::logger::init_logger;
use crate::panic::setup_panic_handler;
use anyhow::bail;
use log::{error, info};
use marston_core::context::Context;
use marston_core::{MPath, MResult};
use std::env;
use std::path::Path;
use std::process::exit;
use marston_core::fs::to_mpath;

mod clap;
mod commands;
mod logger;
mod panic;

fn main() -> MResult<()> {
    let args = cli().try_get_matches().unwrap_or_else(|err| {
        err.print().expect("Error printing error");
        exit(1);
    });
    init_logger()?;
    setup_panic_handler(args.get_flag("no-backtrace"));

    let (cmd, matches) = match args.subcommand() {
        Some((cmd, args)) => (cmd, args),
        None => {
            cli().print_help()?;

            return Ok(());
        }
    };
    let c_dir = env::current_dir()?;
    let context = Context::new(&to_mpath(c_dir)?)?;
    info!("current project: {}", context.name());

    if let Err(err) = execute(context, cmd) {
        error!("{err}");
    }

    Ok(())
}

pub fn execute(ctx: Context, name: &str) -> MResult<()> {
    let cmd = match name {
        "build" => build_command,
        _ => bail!("Unknown command: {name}"),
    };

    cmd(ctx)
}
