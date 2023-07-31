use ffmpeg::util::frame::Video;

use crate::{term::TermSize, utils::Coordinate};

extern crate ffmpeg_next as ffmpeg;

pub enum PointState {
    Changed,
    Same,
}

pub struct AsciiFrame<State = Empty> {
    point_buffer: Vec<AsciiFramePoint>,
    terminal_size: TermSize,
    state: std::marker::PhantomData<State>,
}

/// A single point on the AsciiFrame (an ASCII character),
/// contains additional information about the state of this point
/// changed is true if the character was not the same the frame before,
/// else false
pub struct AsciiFramePoint {
    pub coordinate: Coordinate,
    pub char: char,
}

#[derive(Clone)]
pub struct Frame {
    pub original_frame: Video,
    pub terminal_size: TermSize,
}


pub enum Empty {}
pub enum Full {}

impl AsciiFramePoint {
    pub fn new(coordinate: Coordinate, char: char) -> Self {
        Self { coordinate, char }
    }
}

impl Frame {
    pub fn new(frame: ffmpeg::util::frame::Video) -> Frame {
        let terminal_size = crate::term::get().unwrap(); //TODO update needed!
        Frame {
            original_frame: frame,
            terminal_size,
        }
    }
}

impl AsciiFrame<Empty> {
    pub fn send_char_buffer(&self, char_buffer: Vec<AsciiFramePoint>) -> AsciiFrame<Full> {
        todo!()
    }
}

impl AsciiFrame<Full> {}

impl AsciiFrame {
    pub fn new() -> AsciiFrame<Empty> {
        let terminal_size = crate::term::get().unwrap(); //TODO update needed!
        Self {
            point_buffer: Vec::new(),
            terminal_size,
            state: Default::default(),
        }
    }
}
