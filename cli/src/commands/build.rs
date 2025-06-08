use glob::glob;
use marston_core::{MPath, MResult, context::Context, fs::to_mpath};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn build_command(ctx: Context) -> MResult<()> {
    let pattern = format!("{}/**/*.mr", ctx.main_dir());

    let files: Vec<MPath> = glob(&pattern)?
        .filter_map(Result::ok)
        .filter_map(|path_buf| to_mpath(path_buf).ok())
        .collect();

    let ctx = Arc::new(Mutex::new(ctx));
    files.par_iter().try_for_each(|file| {
        let ctx = ctx.clone();
        let file = file.clone();
        let mut ctx = ctx.lock().unwrap();
        ctx.process_file(&file)
    })?;

    Ok(())
}
