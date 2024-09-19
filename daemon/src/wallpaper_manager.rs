use std::option::Option;
use std::time::{Instant, Duration};
use std::path::PathBuf;
use std::process::Command;
use std::str;

use color_eyre::Result;

pub struct WallpaperManager {
    #[allow(dead_code)]
    pub dir: PathBuf,
    pub interval: Duration,
    pub wallpaper_daemon: WallpaperDaemon,
    pub socket_path: PathBuf,
    pub is_paused: bool,
    pub last_update: Option<Instant>,
    pub last_pause: Option<Instant>,
    pub last_resume: Option<Instant>,
    pub paths: Vec<PathBuf>,
    pub waiting_after_pause: bool,
    pub skip_after_manual: bool,
}

impl WallpaperManager {
    pub fn new(
        dir: PathBuf,
        interval: Duration,
        wallpaper_daemon: WallpaperDaemon,
        socket_path: PathBuf,
    ) -> Result<Self> {
        Ok(Self {
            dir,
            interval,
            wallpaper_daemon,
            socket_path,
            is_paused: false,
            last_update: None,
            last_pause: None,
            last_resume: None,
            paths: Vec::new(),
            waiting_after_pause: false,
            skip_after_manual: false,
        })
    }

    pub fn set_wallpaper(&mut self, path: PathBuf) -> Result<()> {
        self.last_update = Some(Instant::now());

        let daemon = self.wallpaper_daemon.clone();
        let paths = self.paths.clone();

        std::thread::spawn(move || {
            match daemon {
                WallpaperDaemon::Swww => {
                    if let Err(e) = Command::new("swww").arg("img").arg(&path).output() {
                        eprintln!("Failed to execute 'swww img': {:?}", e);
                    }
                    println!("Wallpaper {}", &path.display());
                }
                WallpaperDaemon::Hyprpaper => {
                    if let Err(e) = hyprpaper_preload(&path.to_string_lossy()) {
                        eprintln!("Failed to preload wallpaper: {:?}", e);
                    }

                    if let Err(e) = hyprpaper_wallpaper(&path.to_string_lossy()) {
                        eprintln!("Failed to set wallpaper: {:?}", e);
                    }

                    if let Ok(loaded) = hyprpaper_get_loaded() {
                        let indices = [ 0, 1, paths.len() - 1, 2, 3, paths.len() - 2, 4, paths.len() - 3, 5 ];

                        let needed: Vec<String> = indices
                            .iter()
                            .filter_map(|&i| paths.get(i))
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();

                        for path in needed.iter().filter(|&path| !loaded.contains(path)) {
                            if let Err(e) = hyprpaper_preload(path) {
                                eprintln!("Failed to preload needed wallpaper: {:?}", e);
                            }
                        }

                        for path in loaded.iter().filter(|path| !needed.contains(path)) {
                            if let Err(e) = hyprpaper_unload(path) {
                                eprintln!("Failed to unload wallpaper: {:?}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

pub fn hyprpaper_preload(path: &str) -> Result<()> {
    println!("Preload {}", path);
    Command::new("hyprctl")
        .arg("hyprpaper")
        .arg("preload")
        .arg(path)
        .output()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to execute preload command: {:?}", e))?;
    Ok(())
}

pub fn hyprpaper_wallpaper(path: &str) -> Result<()> {
    println!("Wallpaper {}", path);
    Command::new("hyprctl")
        .arg("hyprpaper")
        .arg("wallpaper")
        .arg(format!(",{}", path))
        .output()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to execute wallpaper command: {:?}", e))?;
    Ok(())
}

pub fn hyprpaper_unload(path: &str) -> Result<()> {
    println!("Unload {}", path);
    Command::new("hyprctl")
        .arg("hyprpaper")
        .arg("unload")
        .arg(path)
        .output()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to execute unload command: {:?}", e))?;
    Ok(())
}

pub fn hyprpaper_get_loaded() -> Result<Vec<String>> {
    let output = Command::new("hyprctl")
        .arg("hyprpaper")
        .arg("listloaded")
        .output()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to execute listloaded command: {:?}", e))?;

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| color_eyre::eyre::eyre!("Invalid UTF-8 sequence: {:?}", e))?;

    Ok(stdout.lines().map(|line| line.to_string()).collect())
}

#[derive(clap::ValueEnum, Clone, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WallpaperDaemon {
    Swww,
    Hyprpaper,
}
