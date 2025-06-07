use fern::Dispatch;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use marston::MResult;

pub fn init_logger() -> MResult<()> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue)
        .trace(Color::BrightBlack);

    Ok(Dispatch::new()
        .format(move |out, message, record| {
            let level = record.level();

            let colored_level = colors.color(level).to_string();
            let colored_level =
                colored_level.chars().map(|c| c.to_ascii_lowercase()).collect::<String>();

            out.finish(format_args!(
                "{colored_level} {message}",
                colored_level = colored_level,
                message = message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?)
}
