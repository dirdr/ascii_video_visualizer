use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use anyhow::Result;

use crate::{args::Arguments, frame::AsciiFrame, GenericSharedQueue};

pub struct Encoder {
    ascii_frame_queue: Arc<GenericSharedQueue<AsciiFrame>>,
    cli: Arguments,
}

impl Encoder {
    pub fn new(ascii_frame_queue: Arc<GenericSharedQueue<AsciiFrame>>, cli: Arguments) -> Self {
        Self {
            ascii_frame_queue,
            cli,
        }
    }

    pub fn start(&self) -> JoinHandle<()> {
        let ascii_frame_queue = Arc::clone(&self.ascii_frame_queue);
        let output_path = self.cli.output_path.clone();

        thread::spawn(move || {})
    }

    pub fn encode_frame() -> Result<()> {
        todo!()
    }
}
