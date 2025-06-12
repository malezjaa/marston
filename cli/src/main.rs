use crate::{
    clap::cli, commands::build::build_command, logger::init_logger, panic::setup_panic_handler,
};
use anyhow::bail;
use log::{error, info};
use marston_core::{MResult, context::Context, fs::to_mpath};
use std::{env, process::exit};

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

    let (cmd, _) = match args.subcommand() {
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
