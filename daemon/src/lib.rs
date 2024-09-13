mod ipc_server;
mod shuffle;
mod socket;
pub mod wallpaper_manager;

use std::path::PathBuf;
use std::time::Duration;

use ipc_server::{handle_message, listen_on_ipc_socket};
use wallpaper_manager_ipc::socket_path;
use color_eyre::{
    eyre::WrapErr,
    Result,
};
use smithay_client_toolkit::reexports::{
    calloop::{self, timer::{Timer, TimeoutAction}},
};

use crate::shuffle::shuffle;
use crate::wallpaper_manager::{WallpaperManager, WallpaperDaemon};

pub fn run(dir: PathBuf, interval: u64, wallpaper_daemon: WallpaperDaemon) -> Result<()> {
    let mut event_loop = calloop::EventLoop::<WallpaperManager>::try_new()?;
    let mut wallpaper_manager = WallpaperManager::new(dir.clone(), Duration::from_millis(interval), wallpaper_daemon, socket_path()?)?;
    
    wallpaper_manager.paths = std::fs::read_dir(dir.clone()).unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>().unwrap();
    shuffle(&mut wallpaper_manager.paths);

    let paths_length = wallpaper_manager.paths.len();
    println!("Total wallpapers: {}", paths_length);

    let socket = listen_on_ipc_socket(&wallpaper_manager.socket_path).context("spawning the ipc socket")?;

    event_loop
        .handle()
        .insert_source(socket, |stream, _, wallpaper_manager| {
            if let Err(err) = handle_message(stream, wallpaper_manager) {
                println!("{:?}", err);
            }
        })?;

    let source = Timer::from_duration(Duration::from_secs(0));

    event_loop
        .handle()
        .insert_source(source, |_event, _metadata, wallpaper_manager| {
            if wallpaper_manager.waiting_after_pause {
                wallpaper_manager.last_pause = None;
                wallpaper_manager.last_resume = None;
                wallpaper_manager.waiting_after_pause = false;
            }
            if wallpaper_manager.is_paused {
                return TimeoutAction::ToDuration(Duration::from_millis(10));
            }
            if wallpaper_manager.skip_after_manual {
                wallpaper_manager.skip_after_manual = false;
                return TimeoutAction::ToDuration((wallpaper_manager.interval) - (wallpaper_manager.last_update.unwrap().elapsed()));
            }
            if wallpaper_manager.last_pause != None {
                let last_pause_clone = wallpaper_manager.last_pause.clone().unwrap();
                let last_resume_clone = wallpaper_manager.last_resume.clone().unwrap();
                wallpaper_manager.waiting_after_pause = true;

                return TimeoutAction::ToDuration(
                    wallpaper_manager.interval
                    - std::time::Instant::duration_since(&std::time::Instant::now(), last_resume_clone)
                    - std::time::Instant::duration_since(&last_pause_clone, wallpaper_manager.last_update.unwrap())
                );
            }

            wallpaper_manager.paths.rotate_left(1);
            wallpaper_manager.set_wallpaper(wallpaper_manager.paths[0].clone());

            TimeoutAction::ToDuration(wallpaper_manager.interval)
        }).unwrap();

    loop {
        event_loop
            .dispatch(None, &mut wallpaper_manager)
            .context("dispatching the event loop")?;
    }
}
