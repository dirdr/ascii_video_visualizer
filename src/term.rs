use terminal_size::{Height, Width};

pub struct TermSize {
    pub width: u32,
    pub height: u32,
}

pub fn get() -> Option<TermSize> {
    if let Some((Width(w), Height(h))) = terminal_size::terminal_size() {
        return Some(TermSize {
            width: w as u32,
            height: h as u32,
        });
    }
    None
}
