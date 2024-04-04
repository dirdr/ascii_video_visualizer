use std::{
    io::{self, BufWriter, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crossterm::{
    cursor::{MoveTo, Show},
    style::{Color, Print, PrintStyledContent, ResetColor, Stylize},
    terminal::Clear,
    QueueableCommand,
};

use crate::{converter::TerminalPixel, frame::AsciiFrame, GenericSharedQueue};
use crate::{frame::Full, queues::FrameType};

use anyhow::Result;

/// The `Player` struct output his content
/// into stdout to be visualized
pub struct Player {
    should_stop: Arc<AtomicBool>,
    delta: u64,
}

impl Player {
    pub fn new(should_stop: Arc<AtomicBool>, frame_rate: usize) -> Self {
        Self {
            should_stop,
            delta: ((1.0 / frame_rate as f64) * 1000.0) as u64,
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        let queue_clone = GenericSharedQueue::<AsciiFrame<Full>>::global(FrameType::Output);
        let delta = self.delta;
        let should_stop = Arc::clone(&self.should_stop);

        stdout.queue(crossterm::cursor::Hide)?;
        info!("delta between frame {delta}");

        let worker = thread::spawn(move || -> Result<()> {
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
                }?;
                if should_stop.load(Ordering::Relaxed) && queue_guard.is_empty() {
                    Self::reset().unwrap();
                    break;
                }
            }
            Ok(())
        });
        worker.join().unwrap()?;
        Ok(())
    }

    fn reset() -> io::Result<()> {
        let mut stdout = std::io::stdout();
        stdout
            .queue(Show)?
            .queue(Clear(crossterm::terminal::ClearType::All))?;
        Ok(())
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
