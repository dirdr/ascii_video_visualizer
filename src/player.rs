use std::{
    io::{self, BufWriter, Write},
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crossterm::{
    cursor::{MoveTo, Show},
    style::{Color, Print, PrintStyledContent, ResetColor, Stylize},
    QueueableCommand,
};

use crate::frame::Full;
use crate::{converter::TerminalPixel, frame::AsciiFrame, GenericSharedQueue};

/// The `Player` struct output his content
/// into stdout to be visualized
pub struct Player {
    ascii_frame_queue: Arc<GenericSharedQueue<AsciiFrame<Full>>>,
    delta: u64,
}

impl Player {
    pub fn new(frame_queue: Arc<GenericSharedQueue<AsciiFrame<Full>>>, frame_rate: usize) -> Self {
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
        for (y, row) in char_buffer.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                Self::print_char(x as u16, y as u16, *pixel)?;
            }
        }
        bw.flush()?;
        Ok(())
    }

    fn print_char(x: u16, y: u16, terminal_pixel: TerminalPixel) -> io::Result<()> {
        let mut stdout = std::io::stdout();
        match terminal_pixel {
            TerminalPixel::Colored(ch, color) => {
                stdout
                    .queue(MoveTo(x, y - 1))?
                    .queue(PrintStyledContent(ch.on(Color::Black).with(Color::Rgb {
                        r: color[0],
                        g: color[1],
                        b: color[2],
                    })))?
                    .queue(ResetColor)?;
            }
            TerminalPixel::Gray(ch) => {
                stdout.queue(MoveTo(x, y - 1))?.queue(Print(ch))?;
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        let mut stdout = std::io::stdout();
        stdout.queue(Show).ok();
    }
}
