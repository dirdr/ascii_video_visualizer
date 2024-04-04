use std::thread;

use anyhow::Result;
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::flag::Flags;
use ffmpeg::software::scaling::Context;
use ffmpeg::util::frame::video::Video;

use crate::args::{Arguments, Mode};
use crate::frame::Frame;
use crate::queues::FrameType;
use crate::GenericSharedQueue;

/// DecoderWrapper wrap around ffmpeg
/// using 'ffmpeg-next' crates
/// the thread read a video and populate a shared queue with frames
pub struct DecoderWrapper {}

impl DecoderWrapper {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let cli = Arguments::global();
        let frame_queue = GenericSharedQueue::<Frame>::global(FrameType::Input);
        let _path = cli.path.clone();
        let mode = cli.mode;
        let mut ictx = input(&format!("./resources/{}", cli.path.clone()))?;
        let worker = thread::spawn(move || -> Result<()> {
            // find the best video flux
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(ffmpeg::Error::StreamNotFound)?;

            let video_stream_index = input.index();

            // find the decoder
            let context_decoder =
                ffmpeg::codec::context::Context::from_parameters(input.parameters());

            let mut decoder = context_decoder?.decoder().video()?;

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
            )?;

            let packets: Vec<_> = ictx.packets().collect();
            for (stream, packet) in packets {
                if stream.index() == video_stream_index {
                    decoder.send_packet(&packet)?;
                    let mut decoded = Video::empty();
                    while decoder.receive_frame(&mut decoded).is_ok() {
                        let mut scaled = Video::empty();
                        scaler.run(&decoded, &mut scaled)?;
                        let frame = Frame::new(scaled.clone());
                        let mut frame_queue_guard = frame_queue
                            .queue
                            .lock()
                            .expect("failed to acquire the frame mutex");
                        frame_queue_guard.push_back(frame);
                        frame_queue.condvar.notify_all();
                    }
                }
            }
            decoder.send_eof()?;
            Ok(())
        });
        worker.join().unwrap()?;
        Ok(())
    }
}
