use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::flag::Flags;
use ffmpeg::software::scaling::Context;
use ffmpeg::util::frame::video::Video;

use crate::frame::Frame;

pub struct DecoderWrapper {
    path: String,
    frame_queue: Arc<Mutex<VecDeque<Frame>>>,
}

impl DecoderWrapper {
    pub fn new(path: &str, frame_queue: Arc<Mutex<VecDeque<Frame>>>) -> Self {
        Self {
            path: path.to_owned(),
            frame_queue,
        }
    }

    pub fn start(&self) {
        let frame_queue = Arc::clone(&self.frame_queue);
        let path = self.path.clone();

        thread::spawn(move || {
            let mut ictx = input(&path).unwrap();

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

            let mut scaler = Context::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                Pixel::GRAY8,
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
                        let frame = Frame::new(scaled);
                        let mut frame_queue = frame_queue.lock().unwrap();
                        frame_queue.push_back(frame);
                    }
                }
            }
            decoder.send_eof().unwrap();
        });
    }

    pub fn get_frames(&self) -> Arc<Mutex<VecDeque<Frame>>> {
        Arc::clone(&self.frame_queue)
    }
}