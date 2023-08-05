use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    ascii_set::AsciiCharset,
    frame::{AsciiFrame, Frame},
    term, SharedAsciiFrameQueue, SharedFrameQueue,
};

use image::{ImageBuffer, RgbImage};

/// a converter takes `Frame` as an input
/// and convert them into `AsciiFrame` depending on the generic `Mode`
/// this process is done in a separate thread.
pub struct Converter {
    frame_queue: Arc<SharedFrameQueue>,
    ascii_frame_queue: Arc<SharedAsciiFrameQueue>,
    set: AsciiCharset,
}

impl Converter {
    pub fn start(&mut self) {
        let frame_queue = Arc::clone(&self.frame_queue);
        let ascii_frame_queue = Arc::clone(&self.ascii_frame_queue);
        thread::spawn(move || {
            let mut frame_queue = frame_queue.queue.lock().unwrap();
            let mut ascii_frame_queue = ascii_frame_queue.queue.lock().unwrap();
            loop {
                match frame_queue.pop_front() {
                    Some(frame) => {
                        let converted = self.convert_frame(frame.clone());
                        ascii_frame_queue.push_back(converted.clone());
                    }
                    None => {
                        // wait at least 'delta' to print at FPS rate
                        // thread::sleep(Duration::from_millis(delta));
                        // queue_guard = frame_queue.condvar.wait(queue_guard).unwrap()
                    }
                }
            }
        });
    }

    fn convert_frame(&self, frame: Frame) -> AsciiFrame {
        let terminal_size = term::get().unwrap();

        let img: RgbImage = ImageBuffer::from_raw(
            frame.frame.width(),
            frame.frame.height(),
            frame.frame.data(0).to_vec(),
        )
        .unwrap();

        let resized_image = image::imageops::resize(
            &img,
            terminal_size.width,
            terminal_size.height / 2, // because an ascii character is in a rectangular shape
            image::imageops::FilterType::Nearest,
        );

        let mut char_buffer = vec![vec![]];
        for (x, y, pixel) in resized_image.enumerate_pixels() {
            // TODO faire le traitement ici
            // TODO update la doc plus a jour avec les différents mode
        }
        for c in 0..terminal_size.width {
            let mut row = vec![];
            for r in 0..terminal_size.height {
                // TODO en fonction du mode (normal ou noir et blanc cela peux varier)
                // ITU-R BT.601-6 Formula to map RGB Pixel to Gray Level
                // Luminance = 0.299R + 0.587G + 0.114B
                let pixel = resized_image.get_pixel(c, r);
                let luminance = pixel.0
                let char = self.map_gray_level_to_ascii(resized_image.get_pixel(c, r).0[0]);
                row.push(char);
            }
            char_buffer.push(row);
        }

        let frame = AsciiFrame::new();
        frame.send_char_buffer(char_buffer);
        frame
    }

    fn map_gray_level_to_ascii(&self, gray_level: u8) -> char {
        let ascii_scale = " .:-=+*#%@";
        let gray_scale = gray_level as f32 / 255.0;
        let index = (gray_scale * (ascii_scale.len() - 1) as f32).round() as usize;
        ascii_scale.chars().nth(index).unwrap()
    }
}

