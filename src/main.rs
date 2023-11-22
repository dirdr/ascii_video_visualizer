use anyhow::Result;
use args::INSTANCE;
use clap::Parser;
use encoder::Encoder;
use queues::{ASCII_FRAME_QUEUE_INSTANCE, INPUT_FRAME_QUEUE_INSTANCE, OUTPUT_FRAME_QUEUE_INSTANCE};
extern crate ffmpeg_next as ffmpeg;
extern crate pretty_env_logger;

mod args;
mod ascii_set;
mod converter;
mod decoder;
mod encoder;
mod frame;
mod player;
mod queues;
mod term;

#[macro_use]
extern crate log;

use crate::args::Arguments;
use crate::queues::GenericSharedQueue;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use converter::{Converter, FrameToAsciiFrameConverter};
use decoder::DecoderWrapper;
use frame::{AsciiFrame, Frame, Full};
use player::Player;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    ffmpeg::init().unwrap();

    let cli = Arguments::parse();

    let _ = INSTANCE.set(cli);

    let _ = INPUT_FRAME_QUEUE_INSTANCE.set(GenericSharedQueue::<Frame>::new());
    let _ = ASCII_FRAME_QUEUE_INSTANCE.set(GenericSharedQueue::<AsciiFrame<Full>>::new());
    let _ = OUTPUT_FRAME_QUEUE_INSTANCE.set(GenericSharedQueue::<Frame>::new());

    let decoder = DecoderWrapper::new();
    let should_stop = Arc::new(AtomicBool::new(false));
    let mut converter = FrameToAsciiFrameConverter::new(Arc::clone(&should_stop));
    let player = Player::new(Arc::clone(&should_stop), 60);
    decoder.start()?;
    converter.start()?;
    match Arguments::get_rendering_mode() {
        args::Output::Play => {
            let mut player = Player::new(Arc::clone(&should_stop), 60);
            player.start()?;
        }
        args::Output::Encode => {
            let encoder = Encoder::new();
            encoder.start()?;
        }
    };
    should_stop.store(true, Ordering::Relaxed);
    Ok(())
}
