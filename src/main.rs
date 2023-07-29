extern crate ffmpeg_next as ffmpeg;

mod converter;
mod decoder;
mod encoder;
mod frame;
mod player;
mod term;
mod utils;

use std::{collections::VecDeque, sync::{Mutex, Arc}, thread, time::Duration};

use clap::Parser;

use crossterm::QueueableCommand;
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

fn main() -> Result<(), ffmpeg::Error> {
    let cli = Cli::parse();

    // let mode: Mode = match &cli.mode[..] {
    //     "mean" => Mode::Mean,
    //     "individual" => Mode::Individual,
    //     _ => Mode::Individual, // default value
    // };

    let path = format!("./resources/{}", cli.path.clone());

    ffmpeg::init()?;

    let mut stdout = std::io::stdout();
    stdout.queue(crossterm::cursor::Hide).ok();

    let shared_queue: Arc<Mutex<VecDeque<Frame>>> = Arc::new(Mutex::new(VecDeque::new()));
    let decoder = DecoderWrapper::new(&path, shared_queue);
    decoder.start();

    let frames = decoder.get_frames();
    thread::spawn(move || {
        loop {
            let len = frames.lock().unwrap().len();
            println!("Frames read: {}", len);
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Attend indéfiniment
    loop {
        thread::sleep(Duration::from_secs(1));
    }
    stdout.queue(crossterm::cursor::Show).ok();
    Ok(())
}
