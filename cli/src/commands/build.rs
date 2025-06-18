use glob::glob;
use marston_core::{
    MPath, MResult,
    context::Context,
    fs::{clear_dir, to_mpath},
};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn build_command(ctx: Context) -> MResult<()> {
    if ctx.build_dir().exists() {
        clear_dir(ctx.build_dir())?;
    }
    fs_err::create_dir_all(ctx.build_dir())?;

    let pattern = format!("{}/**/*", ctx.main_dir());

    let files: Vec<MPath> = glob(&pattern)?
        .filter_map(Result::ok)
        .filter_map(|path_buf| to_mpath(path_buf).ok())
        .collect();

    let ctx = Arc::new(Mutex::new(ctx));
    files.par_iter().try_for_each(|file| {
        let mut ctx = ctx.lock().unwrap();
        if file.extension() != Some("mr") {
            let stripped = file.strip_prefix(ctx.main_dir())?;
            let out = ctx.build_dir().join(stripped);

            fs_err::copy(file, &out)?;
            return Ok(());
        }

        ctx.process_file(file)
    })?;

    Ok(())
}
