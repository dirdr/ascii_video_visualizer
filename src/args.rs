use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "ascii_video_visualizer")]
#[command(author = "Adrien P. <adrien.pelfresne@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "convert mp4 video into ascii visualisation!")]
pub struct Arguments {
    #[arg(short, long, default_value = "maths.mp4")]
    pub path: String,

    #[arg(short, long, default_value = "gray")]
    pub mode: Mode,

    #[arg(short, long, default_value = "low")]
    pub detail_level: DetailLevel,
}

#[derive(Copy, Clone, ValueEnum, Debug, PartialOrd, Eq, PartialEq)]
pub enum Mode {
    #[clap(alias = "gray")]
    Gray,
}

#[derive(Copy, Clone, ValueEnum, Debug, PartialOrd, Eq, PartialEq)]
pub enum DetailLevel {
    #[clap(alias = "basic")]
    Basic,

    #[clap(alias = "detailed")]
    Detailed,
}
