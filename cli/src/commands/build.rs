use glob::glob;
use marston_core::{MPath, MResult, context::Context, fs::to_mpath};

pub fn build_command(ctx: Context) -> MResult<()> {
    let pattern = format!("{}/**/*.mr", ctx.main_dir());

    let files: Vec<MPath> = glob(&pattern)?
        .filter_map(Result::ok)
        .filter_map(|path_buf| to_mpath(path_buf).ok())
        .collect::<Vec<_>>();

    for path in &files {
        println!("Found MR file: {}", path);
    }

    Ok(())
}
