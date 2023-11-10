use core::panic;
use std::fmt::Display;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::flag::Flags;
use ffmpeg::software::scaling::Context;
use ffmpeg::util::frame::video::Video;

use crate::args::{Arguments, Mode};
use crate::frame::Frame;
use crate::queues::FrameType;
use crate::GenericSharedQueue;

/// The decoder is a wrapper around the ffmpeg tools
/// He takes a video as input and decode it into `Frame`
pub struct DecoderWrapper {}

impl DecoderWrapper {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self) -> JoinHandle<()> {
        let cli = Arguments::global();
        let frame_queue = GenericSharedQueue::<Frame>::global(FrameType::Input);
        let path = cli.path.clone();
        let mut ictx = match input(&format!("./resources/{}", cli.path.clone())) {
            Ok(p) => p,
            Err(_) => {
                error!("can't find the input video file {path}");
                panic!();
            }
        };
        let mode = cli.mode.clone();

        thread::spawn(move || {
            // find the best video flux
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(ffmpeg::Error::StreamNotFound)
                .unwrap();

            let video_stream_index = input.index();

            // find the decoder
            let context_decoder =
                ffmpeg::codec::context::Context::from_parameters(input.parameters()).unwrap();
            let mut decoder = context_decoder.decoder().video().unwrap();

            info!(
                "Original Video width: {}, height : {}",
                decoder.width(),
                decoder.height()
            );

            let pixel_mode = match mode {
                Mode::Gray => Pixel::GRAY8,
                Mode::Color => Pixel::RGB24,
            };

            let mut scaler = Context::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                pixel_mode,
                decoder.width(),
                decoder.height(),
                Flags::BILINEAR,
            )
            .unwrap();

            let packets: Vec<_> = ictx.packets().collect();
            for (stream, packet) in packets {
                if stream.index() == video_stream_index {
                    decoder.send_packet(&packet).unwrap();
                    let mut decoded = Video::empty();
                    while decoder.receive_frame(&mut decoded).is_ok() {
                        let mut scaled = Video::empty();
                        scaler.run(&decoded, &mut scaled).unwrap();
                        let frame = Frame::new(scaled.clone());
                        let mut frame_queue_guard = frame_queue.queue.lock().unwrap();
                        frame_queue_guard.push_back(frame);
                        frame_queue.condvar.notify_all();
                    }
                }
            }
            decoder.send_eof().unwrap();
        })
    }
}
