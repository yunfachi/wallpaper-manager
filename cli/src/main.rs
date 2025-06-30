mod opts;

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

use clap::Parser;
use serde::Serialize;
use serde_json::to_string;
use wallpaper_manager_ipc::{socket_path, IpcError, IpcMessage, IpcResponse};
use wallpaper_manager_daemon::run;

use crate::opts::{Opts, SubCmd};

fn main() {
    let args = Opts::parse();

    let msg = match args.subcmd {
        SubCmd::StartDaemon { dir, interval, wallpaper_daemon } => {
            run(dir, interval, wallpaper_daemon).unwrap();
            std::process::exit(0);
        },
        SubCmd::StopDaemon {} => IpcMessage::StopDaemon {},
        SubCmd::PausePlay {} => IpcMessage::PausePlay {},
        SubCmd::ResumePlay {} => IpcMessage::ResumePlay {},
        SubCmd::NextWallpaper {} => IpcMessage::NextWallpaper {},
        SubCmd::PreviousWallpaper {} => IpcMessage::PreviousWallpaper {},
        SubCmd::MoveWallpaperToIndex { path, index } => IpcMessage::MoveWallpaperToIndex { path, index },
        SubCmd::GoToWallpaper { path } => IpcMessage::GoToWallpaper { path },
        SubCmd::AllWallpapers {} => IpcMessage::AllWallpapers {},
        SubCmd::CurrentWallpaper { watch } => IpcMessage::CurrentWallpaper { watch },
        SubCmd::CurrentInterval {} => IpcMessage::CurrentInterval {},
    };

    let mut conn = UnixStream::connect(socket_path().unwrap()).unwrap();
    conn.write_all(&serde_json::to_vec(&msg).unwrap()).unwrap();

    let reader = BufReader::new(conn);

    for line in reader.lines() {
        let line = line.unwrap();
        let res: Result<IpcResponse, IpcError> =
            serde_json::from_str(&line).expect("wallpaper-managers to return a valid json");

        match res {
            Ok(resp) => match resp {
                IpcResponse::Ok => (),
                IpcResponse::AllWallpapers { entries } => {
                    println!("{}", to_string(&entries).expect("wallpaper-managers to return a valid json"))
                },
                IpcResponse::CurrentWallpaper { path } => {
                    println!("{}", path.display())
                },
                IpcResponse::CurrentInterval { is_paused, interval, elapsed } => {
                    #[derive(Serialize)]
                    struct Item {
                        is_paused: bool,
                        interval: u128,
                        elapsed: u128,
                    }
                    println!("{}", to_string(&Item { is_paused, interval, elapsed }).expect("wallpaper-managers to return a valid json"))
                },
            },
            Err(err) => match err {
                IpcError::PathNotAdded { path } => {
                    eprintln!("Path '{}' not added to paths", path.display())
                },
            }
        }
    }
}
