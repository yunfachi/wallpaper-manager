use std::path::PathBuf;

use clap::Parser;

use wallpaper_manager_daemon::wallpaper_manager::WallpaperDaemon;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCmd,
}

#[derive(clap::Subcommand)]
pub enum SubCmd {
    #[clap(visible_alias = "start")]
    #[command(arg_required_else_help = true)]
    StartDaemon {
        #[clap(short, long, required = true)]
        dir: PathBuf,
        #[clap(short, long, required = true)]
        interval: u64,
        #[clap(short, long, required = true)]
        wallpaper_daemon: WallpaperDaemon,
    },
    #[clap(visible_alias = "stop")]
    StopDaemon {},
    #[clap(visible_alias = "pause")]
    PausePlay {},
    #[clap(visible_alias = "resume")]
    ResumePlay {},
    #[clap(visible_alias = "next")]
    NextWallpaper {},
    #[clap(visible_alias = "previous")]
    PreviousWallpaper {},
    #[clap(visible_alias = "move")]
    #[command(arg_required_else_help = true)]
    MoveWallpaperToIndex {
        #[clap(short, long, required = true)]
        path: PathBuf,
        #[clap(short, long, required = true)]
        index: usize,
    },
    #[clap(visible_alias = "goto")]
    #[command(arg_required_else_help = true)]
    GoToWallpaper {
        #[clap(short, long, required = true)]
        path: PathBuf,
    },
    #[clap(visible_alias = "get-all")]
    AllWallpapers {},
    #[clap(visible_alias = "get-interval")]
    CurrentInterval {},
}
