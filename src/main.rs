mod adb;
mod app;
mod buffer;
mod engine;
mod filter;
mod parser;
mod settings;
mod ui;

use app::OhmylogcatApp;
use crate::ui::reset_pointer_shape;
use crossterm::cursor::SetCursorStyle;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, stdout, Write};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static KEYBOARD_ENHANCEMENT_ENABLED: AtomicBool = AtomicBool::new(false);

fn main() -> io::Result<()> {
    install_panic_hook();

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    if try_enable_keyboard_enhancement(&mut out)? {
        KEYBOARD_ENHANCEMENT_ENABLED.store(true, Ordering::SeqCst);
    }
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    restore_terminal(&mut terminal)?;
    result
}

fn try_enable_keyboard_enhancement(out: &mut io::Stdout) -> io::Result<bool> {
    match execute!(
        out,
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        )
    ) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == io::ErrorKind::Unsupported => {
            let _ = writeln!(
                io::stderr(),
                "Warning: keyboard enhancement unavailable in this terminal \
                 (e.g. legacy conhost). Use Windows Terminal for full key support."
            );
            Ok(false)
        }
        Err(err) => Err(err),
    }
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    if KEYBOARD_ENHANCEMENT_ENABLED.load(Ordering::SeqCst) {
        execute!(
            terminal.backend_mut(),
            SetCursorStyle::DefaultUserShape,
            PopKeyboardEnhancementFlags,
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
    } else {
        execute!(
            terminal.backend_mut(),
            SetCursorStyle::DefaultUserShape,
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
    }
    reset_pointer_shape();
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = OhmylogcatApp::new();
    let tick = Duration::from_millis(50);

    loop {
        app.tick();
        terminal.draw(|frame| app.draw(frame))?;

        if crossterm::event::poll(tick)? {
            let event = crossterm::event::read()?;
            app.handle_event(event)?;
        }

        if app.should_quit() {
            app.restore_pointer();
            break;
        }
    }
    Ok(())
}

fn install_panic_hook() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        if KEYBOARD_ENHANCEMENT_ENABLED.load(Ordering::SeqCst) {
            let _ = execute!(
                stdout(),
                SetCursorStyle::DefaultUserShape,
                PopKeyboardEnhancementFlags,
                LeaveAlternateScreen,
                DisableMouseCapture
            );
        } else {
            let _ = execute!(
                stdout(),
                SetCursorStyle::DefaultUserShape,
                LeaveAlternateScreen,
                DisableMouseCapture
            );
        }
        reset_pointer_shape();
        original(info);
    }));
}
