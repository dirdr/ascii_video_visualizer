use std::{
    fmt::Display,
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{
    args::{
        DetailLevel,
        Mode::{self, Color, Gray},
    },
    ascii_set::{BASIC, DETAILED},
    frame::{AsciiFrame, Frame, Full},
    term, SharedAsciiFrameQueue, SharedFrameQueue,
};

use image::{
    DynamicImage, GenericImageView, GrayImage, ImageBuffer, ImageResult, Rgb, RgbImage, Rgba,
};

#[derive(Copy, Clone)]
pub enum TerminalPixel {
    Gray(char),
    Colored(char, Rgba<u8>),
}

/// a converter takes `Frame` as an input
/// and convert them into `AsciiFrame` depending on the generic `Mode`
/// this process is done in a separate thread.
pub struct Converter {
    frame_queue: Arc<SharedFrameQueue>,
    ascii_frame_queue: Arc<SharedAsciiFrameQueue>,
    set: &'static str,
    mode: Mode,
}

impl Converter {
    pub fn new(
        frame_queue: Arc<SharedFrameQueue>,
        ascii_frame_queue: Arc<SharedAsciiFrameQueue>,
        detail_level: DetailLevel,
        mode: Mode,
    ) -> Self {
        Self {
            frame_queue,
            ascii_frame_queue,
            set: match detail_level {
                DetailLevel::Basic => BASIC,
                DetailLevel::Detailed => DETAILED,
            },
            mode,
        }
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        let frame_queue = Arc::clone(&self.frame_queue);
        let ascii_frame_queue = Arc::clone(&self.ascii_frame_queue);
        let set = self.set;
        let mut index = 0;
        let mode = self.mode.clone();
        thread::spawn(move || {
            let mut frame_queue_guard = frame_queue.queue.lock().unwrap();
            loop {
                match frame_queue_guard.pop_front() {
                    Some(frame) => {
                        let converted = Self::convert_frame(frame.clone(), set, mode, &mut index);
                        let mut ascii_frame_queue_guard = ascii_frame_queue.queue.lock().unwrap();
                        ascii_frame_queue_guard.push_back(converted.clone());
                        ascii_frame_queue.condvar.notify_all();
                    }
                    None => {
                        // block the thread until a frame is available in the queue
                        frame_queue_guard = frame_queue.condvar.wait(frame_queue_guard).unwrap();
                    }
                }
            }
        })
    }

    fn convert_frame(
        frame: Frame,
        charset: &'static str,
        mode: Mode,
        index: &mut i32,
    ) -> AsciiFrame<Full> {
        // TODO faire les deux modes (Sampling et Resizing)
        let term_size = term::get().unwrap();

        info!(
            "terminal size from converter : widt={}, height={}",
            term_size.width, term_size.height
        );

        // TODO prendre en compte les ascii rectangulaire
        let image_buffer = match mode {
            Gray => {
                let buffer: Option<GrayImage> = ImageBuffer::from_raw(
                    frame.frame.width(),
                    frame.frame.height(),
                    frame.frame.data(0).to_vec(),
                );
                buffer.map(DynamicImage::ImageLuma8)
            }
            Color => {
                let buffer: Option<RgbImage> = ImageBuffer::from_raw(
                    frame.frame.width(),
                    frame.frame.height(),
                    frame.frame.data(0).to_vec(),
                );
                buffer.map(DynamicImage::ImageRgb8)
            }
        }
        .expect("expected a frame")
        .resize_exact(
            term_size.width,
            term_size.height,
            image::imageops::FilterType::Nearest,
        );

        info!(
            "final buffer size : width={}, height={}",
            image_buffer.width(),
            image_buffer.height()
        );

        //Self::save_frame(image_buffer.clone(), index);

        let mut char_buffer = vec![vec![]];
        for y in 0..image_buffer.height() {
            let mut row = vec![];
            for x in 0..image_buffer.width() {
                let pixel = image_buffer.get_pixel(x, y).clone();
                let char = Self::map_luminance_to_char(pixel[0], charset);
                match mode {
                    Color => row.push(TerminalPixel::Colored(char, pixel)),
                    Gray => row.push(TerminalPixel::Gray(char)),
                }
            }
            char_buffer.push(row);
        }
        AsciiFrame::new().send_char_buffer(char_buffer)
    }

    fn map_luminance_to_char(luminance: u8, charset: &'static str) -> char {
        let gray_scale = (luminance as f64) / 255.0;
        let index = (gray_scale * (charset.len() - 1) as f64).round() as usize;
        charset.chars().nth(index).unwrap()
    }

    fn save_frame(frame: DynamicImage, index: &mut i32) -> ImageResult<()> {
        frame.save(format!("resources/frame_dump/frame{index}.png"))?;
        *index += 1;
        Ok(())
    }
}

impl Display for Converter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let frames = Arc::clone(&self.frame_queue);
        let ascii_frames = Arc::clone(&self.ascii_frame_queue);
        let frames_len = frames.queue.lock().unwrap().len();
        let ascii_frames_len = ascii_frames.queue.lock().unwrap().len();
        write!(
            f,
            "frame_queue {} ascii_frame_queue {}",
            frames_len, ascii_frames_len
        )
    }
}
