use std::{
    fmt::Display,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    frame::{AsciiFrame, Frame, Full},
    term, SharedAsciiFrameQueue, SharedFrameQueue,
};

use ffmpeg::ffi::av_stream_get_codec_timebase;
use image::{GrayImage, ImageBuffer, RgbImage};

/// a converter takes `Frame` as an input
/// and convert them into `AsciiFrame` depending on the generic `Mode`
/// this process is done in a separate thread.
pub struct Converter {
    frame_queue: Arc<SharedFrameQueue>,
    ascii_frame_queue: Arc<SharedAsciiFrameQueue>,
    set: &'static str,
}

impl Converter {
    pub fn new(
        frame_queue: Arc<SharedFrameQueue>,
        ascii_frame_queue: Arc<SharedAsciiFrameQueue>,
        set: &'static str,
    ) -> Self {
        Self {
            frame_queue,
            ascii_frame_queue,
            set,
        }
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        let frame_queue = Arc::clone(&self.frame_queue);
        let ascii_frame_queue = Arc::clone(&self.ascii_frame_queue);
        let set = self.set;
        thread::spawn(move || {
            let mut frame_queue_guard = frame_queue.queue.lock().unwrap();
            loop {
                match frame_queue_guard.pop_front() {
                    Some(frame) => {
                        let converted = Self::convert_frame(frame.clone(), set);
                        let mut ascii_frame_queue_guard = ascii_frame_queue.queue.lock().unwrap();
                        ascii_frame_queue_guard.push_back(converted.clone());
                        ascii_frame_queue.condvar.notify_all();
                    }
                    None => {
                        // block the thread until a frame is avaible in the queue
                        frame_queue_guard = frame_queue.condvar.wait(frame_queue_guard).unwrap();
                    }
                }
            }
        })
    }

    fn convert_frame(frame: Frame, charset: &'static str) -> AsciiFrame<Full> {
        // TODO faire les deux modes (Sampling et Resizing)
        let terminal_size = term::get().unwrap();

        let img: Option<GrayImage> = ImageBuffer::from_raw(
            frame.frame.width(),
            frame.frame.height(),
            frame.frame.data(0).to_vec(),
        );

        let img = match img {
            Some(value) => value,
            None => {
                println!("the recieved Frame is none");
                // TODO a changer et juste sortir de programme c'est deguelasse la
                return AsciiFrame::new().send_char_buffer(vec![]);
            }
        };

        let resized_image = image::imageops::resize(
            &img,
            terminal_size.width,
            terminal_size.height / 2, // because an ascii character is in a rectangular shape
            image::imageops::FilterType::Nearest,
        );

        let mut char_buffer = vec![vec![]];
        let mut row = vec![];
        for (_x, y, pixel) in resized_image.enumerate_pixels() {
            let luminance = pixel[0];
            let char = Self::map_gray_level_to_ascii(luminance, charset);
            row.push(char);
            if y == resized_image.width() {
                char_buffer.push(row.clone());
                row.clear();
            }
        }
        AsciiFrame::new().send_char_buffer(char_buffer)
    }

    fn map_gray_level_to_ascii(luminance: u8, charset: &'static str) -> char {
        let gray_scale = (luminance as f64) / 255.0;
        let index = (gray_scale * (charset.len() - 1) as f64).round() as usize;
        charset.chars().nth(index).unwrap()
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
