use std::{
    collections::VecDeque,
    io::Stdout,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossterm::QueueableCommand;

use crate::{
    frame::{AsciiFrame, Frame},
    SharedAsciiFrameQueue,
};

/// The `Player` struct output his content
/// into stdout to be visualized
pub struct Player {
    ascii_frame_queue: Arc<SharedAsciiFrameQueue>,
    delta: u64,
    stdout: Stdout,
}

impl Player {
    pub fn new(frame_queue: Arc<SharedAsciiFrameQueue>, frame_rate: usize) -> Self {
        Self {
            ascii_frame_queue: frame_queue,
            delta: ((1 / frame_rate) * 1000) as u64,
            stdout: std::io::stdout(),
        }
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        self.stdout.queue(crossterm::cursor::Hide).ok();
        let queue_clone = Arc::clone(&self.ascii_frame_queue);
        let delta = self.delta.clone();
        thread::spawn(move || {
            let mut queue_guard = queue_clone.queue.lock().unwrap();
            loop {
                match queue_guard.pop_front() {
                    Some(frame) => Self::print_frame(frame.clone()),
                    None => {
                        // wait at least 'delta' to print at FPS rate
                        thread::sleep(Duration::from_millis(delta));
                        queue_guard = queue_clone.condvar.wait(queue_guard).unwrap()
                    }
                }
            }
        })
    }

    pub fn print_frame(frame: AsciiFrame) {}

    pub fn stop(&mut self) {
        self.stdout.queue(crossterm::cursor::Show).ok();
    }
}
