use anstyle::{AnsiColor, Effects, Style};
use clap::{Arg, ArgAction, Command, builder::Styles};

pub const NOP: Style = Style::new();
pub const HEADER: Style = AnsiColor::Magenta.on_default().effects(Effects::BOLD);
pub const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
pub const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
pub const WARN: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
pub const NOTE: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const GOOD: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

pub fn opt(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).help(help).action(ArgAction::Set)
}

pub fn positional(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).help(help).index(1)
}

pub const COMPILATION_HEADING: &str = "Compilation options";

pub fn cli() -> Command {
    let styles = {
        Styles::styled()
            .header(HEADER)
            .usage(USAGE)
            .literal(LITERAL)
            .placeholder(PLACEHOLDER)
            .error(ERROR)
            .valid(VALID)
            .invalid(INVALID)
    };

    Command::new("brim")
        .allow_external_subcommands(true)
        .styles(styles)
        .arg(
            opt("verbose", "Use verbose output").short('v').action(ArgAction::SetTrue).global(true),
        )
        .arg(
            opt("no-color", "Disable colored output")
                .long("no-color")
                .action(ArgAction::SetTrue)
                .global(true),
        )
        .arg(
            opt("time", "Prints the time taken to run the project")
                .short('t')
                .action(ArgAction::SetTrue)
                .global(true),
        )
        .arg(
            opt("no-backtrace", "Do not print a backtrace on panic")
                .long("no-backtrace")
                .action(ArgAction::SetTrue)
                .global(true),
        )
}
