use std::collections::VecDeque;

use crate::frame::AsciiFrame;

/// The `Player` struct output his content
/// into stdout (terminal) to be visualized
pub struct Player {
    frame_stack: VecDeque<AsciiFrame>,
    frame_rate: u8,
}

impl Player {
    pub fn new() -> Self {
        todo!()
    }
    pub fn render_frame() {
        todo!()
    }
H
