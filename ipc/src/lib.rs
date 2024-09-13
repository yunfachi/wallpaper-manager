use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xdg::{BaseDirectories, BaseDirectoriesError};

#[derive(Serialize, Deserialize, PartialEq)]
pub enum IpcMessage {
    StopDaemon,
    PausePlay,
    ResumePlay,
    NextWallpaper,
    PreviousWallpaper,
    MoveWallpaperToIndex { path: PathBuf, index: usize },
    GoToWallpaper { path: PathBuf },
    AllWallpapers,
    CurrentInterval,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum IpcResponse {
    Ok,
    AllWallpapers { entries: Vec<PathBuf> },
    CurrentInterval { is_paused: bool, interval: u128, elapsed: u128 },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum IpcError {
    PathNotAdded { path: PathBuf }
}

pub fn socket_path() -> Result<PathBuf, BaseDirectoriesError> {
    let xdg_dirs = BaseDirectories::with_prefix("wallpaper-manager")?;
    Ok(xdg_dirs.get_runtime_directory()?.join("wallpaper-manager.sock"))
}
