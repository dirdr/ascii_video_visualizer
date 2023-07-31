use terminal_size::{Height, Width};

#[derive(Clone)]
pub struct TermSize {
    pub width: u32,
    pub height: u32,
}

pub fn get() -> Option<TermSize> {
    terminal_size::terminal_size().map(|(Width(w), Height(h))| TermSize {
        width: w as u32,
        height: h as u32,
    })
}
