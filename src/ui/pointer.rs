//! Mouse pointer shape via OSC 22 (supported by iTerm2 3.5+, Terminal.app, Kitty, Ghostty, etc.).

use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerShape {
    Default,
    Text,
}

impl PointerShape {
    fn escape(self) -> &'static str {
        match self {
            PointerShape::Default => "\x1b]22;default\x1b\\",
            PointerShape::Text => "\x1b]22;text\x1b\\",
        }
    }
}

pub fn set_pointer_shape(shape: PointerShape) -> io::Result<()> {
    io::stdout().write_all(shape.escape().as_bytes())?;
    io::stdout().flush()
}

pub fn reset_pointer_shape() {
    let _ = set_pointer_shape(PointerShape::Default);
}
