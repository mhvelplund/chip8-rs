use crate::constants::{HEIGHT, WIDTH};
use clap::Parser;
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::{
    ExecutableCommand,
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyModifiers, poll},
    execute,
    style::{Color, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, EnterAlternateScreen, LeaveAlternateScreen, SetSize, size},
};
use std::io::{Write, stdout};
use std::{path::PathBuf, time::Duration};

/// Set up the terminal for the application.
///
/// # Return
/// * `Ok(())` if the terminal was successfully set up.
/// * `Err` if there was an error during the setup process.
pub fn setup_terminal() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    execute!(stdout, Hide)?;
    execute!(stdout, SetSize(WIDTH as u16, (HEIGHT + 2) as u16))?;

    Ok(())
}

pub fn set_styles() -> Result<(), Box<dyn std::error::Error>> {
    // execute!(stdout(),SetBackgroundColor(Color::Yellow))?;
    // execute!(stdout(),SetForegroundColor(Color::Red))?;
    // execute!(stdout(),Clear(terminal::ClearType::All))?;
    Ok(())
}

/// Restore the terminal to its original state.
///
/// # Arguments
/// * `original_size` - A tuple containing the original width and height of the terminal.
///
/// # Return
/// * `Ok(())` if the terminal was successfully restored.
/// * `Err` if there was an error during the restoration process.
pub fn cleanup_terminal(original_size: (u16, u16)) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    execute!(stdout, Show)?;
    execute!(stdout, LeaveAlternateScreen)?;
    execute!(stdout, SetSize(original_size.0, original_size.1))?;
    execute!(stdout, PopKeyboardEnhancementFlags)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

/// Check if the event is an exit command (Esc key or Ctrl+C).
///
/// # Arguments
/// * `event` - A reference to the event to check.
///
/// # Return
/// * `Ok(true)` if the event is an exit command.
/// * `Ok(false)` otherwise.
/// * `Err` if there was an error during the check.
pub fn should_exit(event: &Event) -> Result<bool, Box<dyn std::error::Error>> {
    if let Event::Key(key_event) = event.to_owned()
        && (key_event.code == KeyCode::Esc
            || (key_event.code == KeyCode::Char('c')
                && key_event.modifiers == KeyModifiers::CONTROL))
    {
        Ok(true)
    } else {
        Ok(false)
    }
}
