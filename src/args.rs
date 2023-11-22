use clap::{Parser, ValueEnum};
use once_cell::sync::OnceCell;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "ascii_video_visualizer",
    about = "convert video into ascii visualisation!",
    author = "Adrien P. <adrien.pelfresne@gmail.com>",
    version = "1.1"
)]
pub struct Arguments {
    /// the video path (with file extension)
    #[arg(short, long, default_value = "cat.mp4")]
    pub path: String,
    /// the rendering mode
    #[arg(short, long, default_value = "gray")]
    pub mode: Mode,
    /// the detail level (how many characters are used to render)
    #[arg(short, long, default_value = "basic")]
    pub detail_level: DetailLevel,
    // the optional output path, if doesn't provided, the video will be played in the terminal
    #[arg(short, long)]
    pub output_path: Option<String>,
}

#[derive(Copy, Clone, ValueEnum, Debug, PartialOrd, Eq, PartialEq)]
pub enum Mode {
    #[clap(alias = "gray")]
    Gray,
    #[clap(alias = "color")]
    Color,
}

#[derive(Copy, Clone, ValueEnum, Debug, PartialOrd, Eq, PartialEq)]
pub enum DetailLevel {
    #[clap(alias = "basic")]
    Basic,

    #[clap(alias = "detailed")]
    Detailed,
}

pub enum Output {
    Play,
    Encode,
}

pub static INSTANCE: OnceCell<Arguments> = OnceCell::new();

impl Arguments {
    pub fn get_rendering_mode() -> Output {
        match &Arguments::global().output_path {
            Some(_) => Output::Encode,
            None => Output::Play,
        }
    }
    pub fn global() -> &'static Arguments {
        INSTANCE.get().expect("arguments are not initialized")
    }
}
