//! IPC socket server.
//! Based on <https://github.com/catacombing/catacomb/blob/master/src/ipc_server.rs>

use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::Instant;

use color_eyre::eyre::{ensure, Context};
use color_eyre::{Result, Section};
use wallpaper_manager_ipc::{IpcError, IpcMessage, IpcResponse};

use crate::socket::SocketSource;
use crate::WallpaperManager;

pub fn listen_on_ipc_socket(socket_path: &Path) -> Result<SocketSource> {
    if socket_path.exists() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    let socket = SocketSource::new(listener)?;
    Ok(socket)
}

pub fn handle_message(
    ustream: UnixStream,
    wallpaper_manager: &mut WallpaperManager,
) -> Result<()> {
    const SIZE: usize = 4096;
    let mut buffer = [0; SIZE];

    let mut stream = BufReader::new(&ustream);
    let n = stream
        .read(&mut buffer)
        .context("error while reading line from IPC")?;

    if n == 0 {
        return Ok(());
    }
    ensure!(n != SIZE, "The message received was too big");


    let message: IpcMessage = serde_json::from_slice(&buffer[..n])
        .with_context(|| format!("error while deserializing message {:?}", &buffer[..n]))?;

    let resp: Result<IpcResponse, IpcError> = match message {
        IpcMessage::StopDaemon { } => {
            std::process::exit(0);
        },
        IpcMessage::PausePlay { } => Ok({
            if !wallpaper_manager.is_paused {
                wallpaper_manager.is_paused = true;
                wallpaper_manager.last_pause = Some(Instant::now());
            }
            IpcResponse::Ok
        }),
        IpcMessage::ResumePlay { } => Ok({
            if wallpaper_manager.is_paused {
                wallpaper_manager.is_paused = false;
                wallpaper_manager.last_resume = Some(Instant::now());
            }
            IpcResponse::Ok
        }),
        IpcMessage::NextWallpaper { } => Ok({
            wallpaper_manager.paths.rotate_left(1);
            wallpaper_manager.set_wallpaper(wallpaper_manager.paths[0].clone());
            wallpaper_manager.skip_after_manual = true;
            IpcResponse::Ok
        }),
        IpcMessage::PreviousWallpaper { } => Ok({
            wallpaper_manager.paths.rotate_right(1);
            wallpaper_manager.set_wallpaper(wallpaper_manager.paths[0].clone());
            wallpaper_manager.skip_after_manual = true;
            IpcResponse::Ok
        }),
        IpcMessage::MoveWallpaperToIndex { path, index } => {
            if let Some(prev_index) = wallpaper_manager.paths.iter().position(|x| x == &path) {
                wallpaper_manager.paths.remove(prev_index);
                wallpaper_manager.paths.insert(index, path);

                if index == 0 || prev_index == 0 {
                    wallpaper_manager.set_wallpaper(wallpaper_manager.paths[0].clone());
                    wallpaper_manager.skip_after_manual = true;
                }
        
                Ok(IpcResponse::Ok)
            } else {
                Err(IpcError::PathNotAdded { path })
            }
        },
        IpcMessage::GoToWallpaper { path } => {
            if let Some(index) = wallpaper_manager.paths.iter().position(|x| x == &path) {
                wallpaper_manager.paths.rotate_left(index);
                wallpaper_manager.set_wallpaper(wallpaper_manager.paths[0].clone());
                wallpaper_manager.skip_after_manual = true;
        
                Ok(IpcResponse::Ok)
            } else {
                Err(IpcError::PathNotAdded { path })
            }
        },
        IpcMessage::AllWallpapers => Ok(IpcResponse::AllWallpapers {
            entries: wallpaper_manager.paths.clone()
        }),
        IpcMessage::CurrentInterval => Ok(IpcResponse::CurrentInterval {
            is_paused: wallpaper_manager.is_paused,
            interval: wallpaper_manager.interval.as_millis(),
            elapsed: {
                let last_update = match wallpaper_manager.last_update {
                    Some(x) => x,
                    None => std::time::Instant::now(),
                };
                let last_pause = match wallpaper_manager.last_pause {
                    Some(x) => x,
                    None => std::time::Instant::now(),
                };
                let last_resume = match wallpaper_manager.last_resume {
                    Some(x) => x,
                    None => std::time::Instant::now(),
                };

                (last_pause - last_update).as_millis() + last_resume.elapsed().as_millis()
            }
        }),
        
    };

    let mut stream = BufWriter::new(ustream);
    stream
        .write_all(&serde_json::to_vec(&resp).unwrap())
        .context("unable to write response to the IPC client")
        .suggestion("Probably the client died, try running it again")?;

    Ok(())
}
