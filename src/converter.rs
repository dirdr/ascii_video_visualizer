use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crate::{
    args::Arguments,
    frame::{Frame, Full},
};
use crate::{
    args::{
        DetailLevel,
        Mode::{self, Color, Gray},
    },
    ascii_set::{self},
    frame::AsciiFrame,
    term,
};
use crate::{queues::FrameType, GenericSharedQueue};

use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, ImageResult, RgbImage, Rgba};

#[derive(Copy, Clone)]
pub enum TerminalPixel {
    Gray(char),
    Colored(char, Rgba<u8>),
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

pub trait Converter {
    fn start(&mut self) -> anyhow::Result<JoinHandle<()>>;
}

pub struct FrameToAsciiFrameConverter {
    should_stop: Arc<AtomicBool>,
}

pub struct AsciiFrameToFrameConverter {}

impl Converter for FrameToAsciiFrameConverter {
    fn start(&mut self) -> anyhow::Result<JoinHandle<()>> {
        let frame_queue = GenericSharedQueue::<Frame>::global(FrameType::Input);
        let output_frame_queue = GenericSharedQueue::<AsciiFrame<Full>>::global(FrameType::Output);
        let mode = Arguments::global().mode.clone();
        let detail_level = Arguments::global().detail_level.clone();
        //TODO changer pour eviter de cloner a chaque fois
        let mut charset_mapper = CharsetMapper::new(match detail_level {
            DetailLevel::Basic => ascii_set::BASIC,
            DetailLevel::Detailed => ascii_set::DETAILED,
        })
        .clone();
        let should_stop = Arc::clone(&self.should_stop);
        Ok(thread::spawn(move || {
            let mut input_queue_guard = frame_queue.queue.lock().unwrap();
            loop {
                match input_queue_guard.pop_front() {
                    Some(frame) => {
                        let converted =
                            Self::convert_frame(frame.clone(), mode, &mut charset_mapper);
                        let mut output_frame_queue_guard = output_frame_queue.queue.lock().unwrap();
                        output_frame_queue_guard.push_back(converted.clone());
                        output_frame_queue.condvar.notify_all();
                    }
                    None => {
                        // block the thread until a frame is available in the queue
                        input_queue_guard = frame_queue.condvar.wait(input_queue_guard).unwrap();
                    }
                }
                if should_stop.load(Ordering::Relaxed) && input_queue_guard.is_empty() {
                    break;
                }
            }
        }))
    }
}

impl FrameToAsciiFrameConverter {
    pub fn new(should_stop: Arc<AtomicBool>) -> Self {
        Self { should_stop }
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

// impl Display for dyn Converter {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let frames = Arc::clone(&self.input_queue);
//         let output_queue = Arc::clone(&self.output_queue);
//         let frames_len = frames.queue.lock().unwrap().len();
//         let ascii_frames_len = ascii_frames.queue.lock().unwrap().len();
//         write!(
//             f,
//             "frame_queue {} ascii_frame_queue {}",
//             frames_len, ascii_frames_len
//         )
//     }
// }
