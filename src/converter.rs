use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    frame::{AsciiFrame, AsciiFramePoint, Frame},
    utils::Coordinate,
    SharedAsciiFrameQueue, SharedFrameQueue,
};

use image::{imageops::resize, GrayImage, ImageBuffer};

#[derive(Clone)]
struct Mean {}

#[derive(Clone)]
struct Individual {}

pub trait ConverterType {
    fn convert_frame(&self, frame: Frame) -> AsciiFrame;
}

impl ConverterType for Mean {
    fn convert_frame(&self, frame: Frame) -> AsciiFrame {
        todo!()
    }
}

impl ConverterType for Individual {
    fn convert_frame(&self, frame: Frame) -> AsciiFrame {
        todo!()
        // let img: GrayImage = ImageBuffer::from_raw(
        //     frame.original_frame.width(),
        //     frame.original_frame.height(),
        //     frame.original_frame.data(0).to_vec(),
        // )
        // .unwrap();
        //
        // let small_img = resize(
        //     &img,
        //     frame.terminal_size.width,
        //     frame.terminal_size.height,
        //     image::imageops::FilterType::Nearest,
        // );
        //
        // let mut point_buff = Vec::new();
        // for c in 0..frame.terminal_size.width {
        //     for r in 0..frame.terminal_size.height {
        //         let char = map_gray_level_to_ascii(small_img.get_pixel(c, r).0[0]);
        //         point_buff.push(AsciiFramePoint::new(Coordinate::new(c, r), char));
        //     }
        // }
        //
        // AsciiFrame::new(point_buff)
    }
}

/// a converter takes `Frame` as an input
/// and convert them into `AsciiFrame` depending on the generic `Mode`
/// this process is done in a separate thread.
pub struct Converter<M: ConverterType> {
    frame_queue: Arc<SharedFrameQueue>,
    ascii_frame_queue: Arc<SharedAsciiFrameQueue>,
    converter_impl: Arc<M>,
}

// impl Converter<Mean> {}
//
// impl Converter<Individual> {}
//
impl<M: ConverterType + Send + Sync + 'static> Converter<M> {
    pub fn start(&mut self) {
        let frame_queue = Arc::clone(&self.frame_queue);
        let ascii_frame_queue = Arc::clone(&self.ascii_frame_queue);
        let converter_impl = Arc::clone(&self.converter_impl);
        thread::spawn(move || {
            let mut frame_queue = frame_queue.queue.lock().unwrap();
            let mut ascii_frame_queue = ascii_frame_queue.queue.lock().unwrap();
            loop {
                match frame_queue.pop_front() {
                    Some(frame) => {
                        let converted = converter_impl.convert_frame(frame.clone());
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
}

fn map_gray_level_to_ascii(gray_level: u8) -> char {
    let ascii_scale = " .:-=+*#%@";
    let gray_scale = gray_level as f32 / 255.0;
    let index = (gray_scale * (ascii_scale.len() - 1) as f32).round() as usize;
    ascii_scale.chars().nth(index).unwrap()
}
