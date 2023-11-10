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
    GenericSharedQueue,
};

pub struct Encoder {
    ascii_frame_queue: Arc<GenericSharedQueue<AsciiFrame<Full>>>,
    cli: Arguments,
}

impl Encoder {
    pub fn new(
        ascii_frame_queue: Arc<GenericSharedQueue<AsciiFrame<Full>>>,
        cli: Arguments,
    ) -> Self {
        Self {
            ascii_frame_queue,
            cli,
        }
    }

    pub fn start(&self) -> JoinHandle<()> {
        let ascii_frame_queue = Arc::clone(&self.ascii_frame_queue);
        let output_path = self.cli.output_path.clone().unwrap();
        let font_data = include_bytes!("../resources/fonts/hack.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing font");
        let scale = Scale::uniform(12.0);
        let v_metrics = font.v_metrics(scale);
        thread::spawn(move || {
            let mut queue_guard = ascii_frame_queue.queue.lock().unwrap();
            loop {
                match queue_guard.pop_front() {
                    Some(frame) => Self::encode_frame(frame, font.clone(), &output_path),
                    None => {
                        // wait unitil a frame is avaible
                        queue_guard = ascii_frame_queue.condvar.wait(queue_guard).unwrap();
                        Ok(())
                    }
                }
                .unwrap_or_else(|e| error!("{e}"));
            }
        })
    }

    pub fn encode_frame(
        frame: AsciiFrame<Full>,
        font: Font,
        output_path: &str,
    ) -> Result<(), Error> {
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
