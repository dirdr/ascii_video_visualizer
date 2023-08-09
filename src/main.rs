extern crate ffmpeg_next as ffmpeg;
extern crate pretty_env_logger;

mod ascii_set;
mod converter;
mod decoder;
mod encoder;
mod frame;
mod player;
mod term;

#[macro_use]
extern crate log;

use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use clap::Parser;

use converter::Converter;
use decoder::DecoderWrapper;
use frame::{AsciiFrame, Frame, Full};
use player::Player;

#[derive(Parser, Debug)]
#[command(name = "ascii_video_visualizer")]
#[command(author = "Adrien P. <adrien.pelfresne@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "convert mp4 video into ascii visualisation!")]
pub struct Cli {
    #[arg(short, long, default_value = "maths.mp4")]
    pub path: String,
    // pub mode: String,
}

// TODO faire une structure générique SharedQueue<T> avec T: Frame
// et les strucutres AsciiFrame qui dérive ce trait et Frame qui dérive ce trait

/// SharedFrameQueue will be shared between a Decoder (Producer) and the Converter (consumer)
pub struct SharedFrameQueue {
    queue: Mutex<VecDeque<Frame>>,
    condvar: Condvar,
}

/// SharedFrameQueue will be shared between a Converter (Producer) and a generic output (consumer).
/// the generic can be a Encoder, or a Player
pub struct SharedAsciiFrameQueue {
    queue: Mutex<VecDeque<AsciiFrame<Full>>>,
    condvar: Condvar,
}

impl SharedFrameQueue {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
        }
    }
}

impl SharedAsciiFrameQueue {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
        }
    }
}

fn main() -> Result<(), ffmpeg::Error> {
    pretty_env_logger::init();
    ffmpeg::init().unwrap();

    let cli = Cli::parse();

    // let mode: Mode = match &cli.mode[..] {
    //     "mean" => Mode::Mean,
    //     "individual" => Mode::Individual,
    //     _ => Mode::Individual, // default value
    // };

    let path = format!("./resources/{}", cli.path.clone());

    let shared_frame_queue = Arc::new(SharedFrameQueue::new());
    let shared_ascii_frame_queue = Arc::new(SharedAsciiFrameQueue::new());

    let decoder = DecoderWrapper::new(&path, Arc::clone(&shared_frame_queue));
    let mut converter = Converter::new(
        Arc::clone(&shared_frame_queue),
        Arc::clone(&shared_ascii_frame_queue),
        ascii_set::BASIC,
    );
    let mut player = Player::new(Arc::clone(&shared_ascii_frame_queue), 60);
    let mut handles = vec![];
    handles.push(decoder.start());
    handles.push(converter.start());
    handles.push(player.start());

    loop {
        thread::sleep(Duration::from_secs(1));
    }

    //
    // print!("{}", converter);
    //
    // Ok(())
}
