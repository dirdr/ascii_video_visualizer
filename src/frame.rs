use std::marker::PhantomData;

use ffmpeg::util::frame::Video;

extern crate ffmpeg_next as ffmpeg;

#[derive(Clone)]
pub enum Empty {}

#[derive(Clone)]
pub enum Full {}

/// width and height are expressed as numeber of character
#[derive(Clone)]
pub struct AsciiFrame<State = Empty> {
    char_buffer: Vec<Vec<char>>,
    state: std::marker::PhantomData<State>,
}

#[derive(Clone)]
pub struct Frame {
    pub frame: Video,
}

impl Frame {
    pub fn new(frame: ffmpeg::util::frame::Video) -> Frame {
        Frame { frame }
    }
}

impl AsciiFrame<Empty> {
    pub fn send_char_buffer(&self, char_buffer: Vec<Vec<char>>) -> AsciiFrame<Full> {
        AsciiFrame {
            char_buffer,
            state: PhantomData,
        }
    }
}

impl AsciiFrame<Full> {
    pub fn get_buffer(&self) -> Vec<Vec<char>> {
        self.char_buffer.clone()
    }
}

impl AsciiFrame {
    pub fn new() -> AsciiFrame<Empty> {
        Self {
            char_buffer: vec![vec![]],
            state: PhantomData,
        }
    }
}
