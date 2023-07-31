extern crate ffmpeg_next as ffmpeg;

mod converter;
mod decoder;
mod encoder;
mod frame;
mod player;
mod term;
mod utils;

use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

use clap::Parser;

use decoder::DecoderWrapper;
use frame::Frame;

#[derive(Parser, Debug)]
#[command(name = "ascii_video_visualizer")]
#[command(author = "Adrien P. <adrien.pelfresne@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "convert mp4 video into ascii visualisation!")]
pub struct Cli {
    #[arg(short, long, default_value = "drift.mp4")]
    pub path: String,
    // pub mode: String,
}

pub struct SharedQueue {
    queue: Mutex<VecDeque<Frame>>,
    condvar: Condvar,
}

impl SharedQueue {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
        }
    }
}

fn main() -> Result<(), ffmpeg::Error> {
    let cli = Cli::parse();

    // let mode: Mode = match &cli.mode[..] {
    //     "mean" => Mode::Mean,
    //     "individual" => Mode::Individual,
    //     _ => Mode::Individual, // default value
    // };

    let path = format!("./resources/{}", cli.path.clone());
    let shared_queue = Arc::new(SharedQueue::new());
    let decoder = DecoderWrapper::new(&path, shared_queue);
    decoder.start();
    let frames = decoder.get_frames();

    Ok(())
}
