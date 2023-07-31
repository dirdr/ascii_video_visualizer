use crate::{
    frame::{AsciiFrame, AsciiFramePoint, Frame},
    utils::Coordinate,
};

use image::{imageops::resize, GrayImage, ImageBuffer};

pub enum Mean {}
pub enum Individual {}

pub trait Mode {
    fn convert_frame(&self) -> AsciiFrame;
}

impl Mode for Mean {
    fn convert_frame(&self) -> AsciiFrame {
        todo!()
    }
}

impl Mode for Individual {
    fn convert_frame(&self) -> AsciiFrame {
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
// pub struct Converter<M: Mode> {
//     mode: M,
// }

// impl Converter<Mean> {}
//
// impl Converter<Individual> {}
//
impl<M: Mode> Converter<M> {

}

fn map_gray_level_to_ascii(gray_level: u8) -> char {
    let ascii_scale = " .:-=+*#%@";
    let gray_scale = gray_level as f32 / 255.0;
    let index = (gray_scale * (ascii_scale.len() - 1) as f32).round() as usize;
    ascii_scale.chars().nth(index).unwrap()
}
