use std::{
    io::{self, BufWriter, Cursor, Stdout, StdoutLock, Write},
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crossterm::{
    style::{ResetColor, SetBackgroundColor, SetForegroundColor, Stylize},
    ExecutableCommand, QueueableCommand,
};

use crate::{converter::TerminalPixel, frame::AsciiFrame};
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
        let mut stdout = std::io::stdout();
        let mut bw = BufWriter::new(stdout.lock());
        let char_buffer = frame.get_buffer();
        for (y, row) in char_buffer.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                match pixel {
                    TerminalPixel::Colored(ch, color) => {
                        stdout
                            .queue(crossterm::cursor::MoveTo(x as u16, y as u16))?
                            .queue(crossterm::style::PrintStyledContent(
                                ch.on(crossterm::style::Color::Black).with(
                                    crossterm::style::Color::Rgb {
                                        r: color[0],
                                        g: color[1],
                                        b: color[2],
                                    },
                                ),
                            ))?
                            .queue(crossterm::style::ResetColor)?;
                    }
                    TerminalPixel::Gray(ch) => {
                        stdout
                            .queue(crossterm::cursor::MoveTo(x as u16, y as u16))?
                            .queue(crossterm::style::Print(ch))?;
                    }
                }
            }
        }
        bw.flush()?;
        Ok(())
    }

    pub fn stop(&mut self) {
        let mut stdout = std::io::stdout();
        stdout.queue(crossterm::cursor::Show).ok();
    }

    fn display_pixel(pixel: TerminalPixel, bw: &mut BufWriter<StdoutLock>) -> io::Result<()> {
        let mut stdout = std::io::stdout();
        Ok(())
    }
}
