use std::{
    io::{self, BufWriter, Write},
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crossterm::QueueableCommand;

use crate::frame::AsciiFrame;
use crate::{frame::Full, SharedAsciiFrameQueue};

/// The `Player` struct output his content
/// into stdout to be visualized
pub struct Player {
    ascii_frame_queue: Arc<SharedAsciiFrameQueue>,
    delta: u64,
}

impl Player {
    pub fn new(frame_queue: Arc<SharedAsciiFrameQueue>, frame_rate: usize) -> Self {
        Self {
            ascii_frame_queue: frame_queue,
            delta: ((1.0 / frame_rate as f64) * 1000.0) as u64,
        }
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        let mut stdout = std::io::stdout();
        stdout.queue(crossterm::cursor::Hide).ok();
        let queue_clone = Arc::clone(&self.ascii_frame_queue);
        let delta = self.delta.clone();
        info!("delta between frame {delta}");
        thread::spawn(move || {
            let mut queue_guard = queue_clone.queue.lock().unwrap();
            loop {
                thread::sleep(Duration::from_millis(delta));
                match queue_guard.pop_front() {
                    Some(frame) => Self::print_frame(frame),
                    None => {
                        // wait at least 'delta' to print at FPS rate
                        queue_guard = queue_clone.condvar.wait(queue_guard).unwrap();
                        Ok(())
                    }
                }
                .unwrap_or_else(|e| error!("{e}"));
            }
        })
    }

    pub fn print_frame(frame: AsciiFrame<Full>) -> io::Result<()> {
        let stdout = std::io::stdout();
        let mut bw = BufWriter::new(stdout.lock());
        let char_buffer = frame.get_buffer();
        for row in char_buffer {
            for char in row {
                write!(bw, "{}", char)?;
            }
            writeln!(bw)?;
        }
        bw.flush()?;
        Ok(())
    }

    pub fn stop(&mut self) {
        let mut stdout = std::io::stdout();
        stdout.queue(crossterm::cursor::Show).ok();
    }
}
