use std::option::Option;
use std::time::{Instant, Duration};
use std::path::PathBuf;

use color_eyre::Result;

pub struct WallpaperManager {
    #[allow(dead_code)]
    pub dir: PathBuf,
    pub interval: Duration,
    pub wallpaper_daemon: WallpaperDaemon,
    pub is_paused: bool,
    pub last_update: Option<Instant>,
    pub last_pause: Option<Instant>,
    pub last_resume: Option<Instant>,
    pub paths: Vec<PathBuf>,
    pub waiting_after_pause: bool,
    pub skip_after_manual: bool,
}

impl WallpaperManager {
    pub fn new(dir: PathBuf, interval: Duration, wallpaper_daemon: WallpaperDaemon) -> Result<Self> {
        Ok(Self {
            dir,
            interval,
            wallpaper_daemon,
            is_paused: false,
            last_update: None,
            last_pause: None,
            last_resume: None,
            paths: [].to_vec(),
            waiting_after_pause: false,
            skip_after_manual: false,
        })
    }

    pub fn set_wallpaper(&mut self, path: PathBuf) {
        self.last_update = Some(std::time::Instant::now());
        std::thread::spawn(|| {
            match self.wallpaper_daemon {
                WallpaperDaemon::Swww => {
                    std::process::Command::new("swww").arg("img").arg(path).output().expect("failed to execute command");
                    // let mut conn = std::os::unix::net::UnixStream::connect("/run/user/1000/swww-wayland-1.socket").unwrap();
                    // conn.write_all(&serde_json::to_vec(&msg).unwrap()).unwrap();
                },
            };
        });
    }
}


#[derive(clap::ValueEnum, Clone, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WallpaperDaemon {
    Swww
}
