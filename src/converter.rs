use std::{
    collections::HashMap,
    fmt::Display,
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{
    args::{
        Arguments, DetailLevel,
        Mode::{self, Color, Gray},
    },
    ascii_set::{BASIC, DETAILED},
    frame::{AsciiFrame, Frame, Full},
    term, GenericSharedQueue,
};

use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, ImageResult, RgbImage, Rgba};

#[derive(Copy, Clone)]
pub enum TerminalPixel {
    Gray(char),
    Colored(char, Rgba<u8>),
}

/// a converter takes `Frame` as an input
/// and convert them into `AsciiFrame` depending on the generic `Mode`
/// this process is done in a separate thread.
pub struct Converter {
    frame_queue: Arc<GenericSharedQueue<Frame>>,
    ascii_frame_queue: Arc<GenericSharedQueue<AsciiFrame<Full>>>,
    charset_mapper: CharsetMapper,
    cli: Arguments,
}

#[derive(Debug, Clone)]
pub struct CharsetMapper {
    charset: Vec<char>,
    cache: HashMap<u8, char>,
}

impl CharsetMapper {
    pub fn new(charset: &'static str) -> Self {
        Self {
            charset: charset.chars().collect(),
            cache: HashMap::new(),
        }
    }

    pub fn map_luminance_to_char(&mut self, luminance: u8) -> char {
        *self.cache.entry(luminance).or_insert_with(|| {
            let gray_scale = (luminance as f64) / 255.0;
            let index = (gray_scale * (self.charset.len() - 1) as f64).round() as usize;
            self.charset[index]
        })
    }
}

impl Converter {
    pub fn new(
        frame_queue: Arc<GenericSharedQueue<Frame>>,
        ascii_frame_queue: Arc<GenericSharedQueue<AsciiFrame<Full>>>,
        cli: Arguments,
    ) -> Self {
        Self {
            frame_queue,
            ascii_frame_queue,
            charset_mapper: CharsetMapper::new(match cli.detail_level {
                DetailLevel::Basic => BASIC,
                DetailLevel::Detailed => DETAILED,
            }),
            cli,
        }
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        let frame_queue = Arc::clone(&self.frame_queue);
        let ascii_frame_queue = Arc::clone(&self.ascii_frame_queue);
        let mut charset_mapper = self.charset_mapper.clone();
        let mode = self.cli.mode.clone();
        thread::spawn(move || {
            let mut frame_queue_guard = frame_queue.queue.lock().unwrap();
            loop {
                match frame_queue_guard.pop_front() {
                    Some(frame) => {
                        let converted =
                            Self::convert_frame(frame.clone(), mode, &mut charset_mapper);
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

    /// process a ffmpeg frame into a image buffer
    /// and resize to match terminal size
    fn process_frame(frame: Frame, mode: &Mode) -> Option<DynamicImage> {
        let terminal_size = term::get().unwrap();
        let image = match mode {
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
        }?
        .resize_exact(
            terminal_size.width,
            terminal_size.height,
            image::imageops::FilterType::Nearest,
        );
        Some(image)
    }

    fn convert_frame(
        frame: Frame,
        mode: Mode,
        charset_mapper: &mut CharsetMapper,
    ) -> AsciiFrame<Full> {
        let image = match Self::process_frame(frame, &mode) {
            Some(b) => b,
            None => {
                error!("the image buffer is None");
                DynamicImage::default()
            }
        };
        let mut char_buffer = vec![vec![]];

        for y in 0..image.height() {
            let mut row = vec![];
            for x in 0..image.width() {
                let pixel = image.get_pixel(x, y);
                let char = charset_mapper.map_luminance_to_char(pixel[0]);
                match mode {
                    Color => row.push(TerminalPixel::Colored(char, pixel)),
                    Gray => row.push(TerminalPixel::Gray(char)),
                }
            }
            char_buffer.push(row);
        }
        AsciiFrame::new().send_char_buffer(char_buffer)
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
