use std::marker::PhantomData;

use ffmpeg::util::frame::Video;

use crate::converter::TerminalPixel;

extern crate ffmpeg_next as ffmpeg;

#[derive(Clone)]
pub enum Empty {}

#[derive(Clone)]
pub enum Full {}

/// width and height are expressed as number of character
#[derive(Clone)]
pub struct AsciiFrame<State = Empty> {
    char_buffer: Vec<Vec<TerminalPixel>>,
    state: std::marker::PhantomData<State>,
}

#[derive(Clone)]
pub struct Frame {
    pub frame: Video,
}

impl Frame {
    pub fn new(frame: ffmpeg::util::frame::Video) -> Self {
        Frame { frame }
    }
}

impl AsciiFrame<Empty> {
    pub fn send_char_buffer(&self, char_buffer: Vec<Vec<TerminalPixel>>) -> AsciiFrame<Full> {
        AsciiFrame {
            char_buffer,
            state: PhantomData,
        }
    }
}

impl AsciiFrame<Full> {
    pub fn get_buffer(&self) -> Vec<Vec<TerminalPixel>> {
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
