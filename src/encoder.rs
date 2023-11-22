use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use anyhow::{Error, Result};
use ffmpeg::{codec, format};
use image::{ImageBuffer, Rgba};
use rusttype::{Font, Scale};

use crate::{
    args::Arguments,
    converter::TerminalPixel,
    frame::{AsciiFrame, Full},
    queues::FrameType,
    GenericSharedQueue,
};

pub struct Encoder {
    font: Font<'static>,
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            font: {
                let font_data = include_bytes!("../resources/fonts/hack.ttf");
                Font::try_from_bytes(font_data as &[u8]).expect("Error constructing font")
            },
        }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let ascii_frame_queue = GenericSharedQueue::<AsciiFrame<Full>>::global(FrameType::Output);
        let cli = Arguments::global();
        let scale = Scale::uniform(12.0);
        let v_metrics = self.font.v_metrics(scale);
        let font = self.font.clone();
        let worker = thread::spawn(move || -> Result<()> {
            // TODO ad should stop to stop the frame is the program is finished
            // use channel to tell the channel to stop
            // https://stackoverflow.com/questions/26199926/how-to-terminate-or-suspend-a-rust-thread-from-another-thread
            let mut queue_guard = ascii_frame_queue.queue.lock().unwrap();
            loop {
                match queue_guard.pop_front() {
                    // unwrap the value because the Encoder is created only if a output path is
                    // specifie, we we know it is not None
                    Some(frame) => {
                        Self::encode_frame(frame, &font, &cli.output_path.as_ref().unwrap())?;
                    }
                    None => {
                        // wait unitil a frame is avaible
                        queue_guard = ascii_frame_queue.condvar.wait(queue_guard).unwrap();
                    }
                };
            }
        });
        worker.join().unwrap()?;
        Ok(())
    }

    pub fn encode_frame(
        frame: AsciiFrame<Full>,
        font: &Font,
        output_path: &str,
    ) -> anyhow::Result<()> {
        let scale = Scale { x: 1.0, y: 1.0 };
        let buffer = frame.get_buffer();
        let img_width = (buffer.len() as f32 * scale.x) as u32;
        let img_height = (buffer[0].len() as f32 * scale.y) as u32;
        let mut img = ImageBuffer::new(img_width, img_height);

        for (y, row) in buffer.into_iter().enumerate() {
            for (x, tp) in row.into_iter().enumerate() {
                let offset_x = (x as u32) * scale.x as u32;
                let offset_y = (y as u32) * scale.y as u32;
                match tp {
                    TerminalPixel::Gray(ch) | TerminalPixel::Colored(ch, _) => {
                        imageproc::drawing::draw_text_mut(
                            &mut img,
                            match tp {
                                TerminalPixel::Gray(_) => Rgba([0, 0, 0, 255]),
                                TerminalPixel::Colored(_, color) => color,
                            },
                            offset_x as i32,
                            offset_y as i32,
                            scale,
                            &font,
                            ch.to_string().as_str(),
                        );
                    }
                }
            }
        }
        let mut output_context = format::output(&output_path)?;
        let bcodec = codec::encoder::find(codec::Id::H264).unwrap();
        Ok(())
    }
}
