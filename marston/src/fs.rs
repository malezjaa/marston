use crate::{MPath, MResult};
use anyhow::anyhow;
use fs_err::read_to_string;
use std::path::PathBuf;

pub fn walk_for_file(mut dir: PathBuf, file_name: &str) -> Option<PathBuf> {
    loop {
        let file_path = dir.join(file_name);
        if file_path.exists() {
            return Some(file_path);
        }

        if !dir.pop() {
            break;
        }
    }

    None
}

/// Convert PathBuf to Utf8PathBuf
pub fn to_mpath(path_buf: PathBuf) -> MResult<MPath> {
    MPath::from_path_buf(path_buf).map_err(|_| anyhow!("Failed to convert path to utf8"))
}

pub fn read_string(path: &MPath) -> MResult<String> {
    read_to_string(path.clone().into_std_path_buf()).map_err(|err| anyhow!(err))
}

pub fn clear_dir(path: &MPath) -> MResult<()> {
    fs_err::remove_dir_all(path.clone().into_std_path_buf()).map_err(|err| anyhow!(err))
}
