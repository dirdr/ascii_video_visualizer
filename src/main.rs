use clap::Parser;
extern crate ffmpeg_next as ffmpeg;
extern crate pretty_env_logger;

mod args;
mod ascii_set;
mod converter;
mod decoder;
mod encoder;
mod frame;
mod player;
mod term;

#[macro_use]
extern crate log;

use crate::args::Arguments;

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Condvar, Mutex,
    },
    thread,
    time::Duration,
};

use converter::Converter;
use decoder::DecoderWrapper;
use frame::{AsciiFrame, Frame, Full};
use player::Player;

pub struct GenericSharedQueue<T> {
    queue: Mutex<VecDeque<T>>,
    condvar: Condvar,
}

impl<T> GenericSharedQueue<T> {
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

    let cli = Arguments::parse();
    let shared_frame_queue = Arc::new(GenericSharedQueue::<Frame>::new());
    let shared_ascii_frame_queue = Arc::new(GenericSharedQueue::<AsciiFrame<Full>>::new());
    let decoder = DecoderWrapper::new(Arc::clone(&shared_frame_queue), cli.clone());
    let should_stop = Arc::new(AtomicBool::new(false));
    let mut converter = Converter::new(
        Arc::clone(&shared_frame_queue),
        Arc::clone(&shared_ascii_frame_queue),
        Arc::clone(&should_stop),
        cli.clone(),
    );
    let mut player = Player::new(
        Arc::clone(&shared_ascii_frame_queue),
        Arc::clone(&should_stop),
        60,
    );

    let decoder_handle = decoder.start();
    let converter_handle = converter.start();
    let player_handle = player.start();
    decoder_handle.join().unwrap();
    should_stop.store(true, Ordering::Relaxed);
    converter_handle.join().unwrap();
    player_handle.unwrap().join().unwrap();
    Ok(())
}
