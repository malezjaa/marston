use crate::error::MResult;
use crate::logger::init_logger;
use crate::panic::setup_panic_handler;

mod error;
mod logger;
mod panic;

fn main() -> MResult<()> {
    init_logger()?;
    setup_panic_handler(false);

    Ok(())
}
