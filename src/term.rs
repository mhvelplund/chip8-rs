use std::{
    io::{Stdout, stdout},
    path::PathBuf,
    time::Duration,
};

use crate::constants::{HEIGHT, WIDTH};
use clap::Parser;

use crossterm::{
    ExecutableCommand,
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyModifiers, poll},
    style::{Color, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, EnterAlternateScreen, LeaveAlternateScreen, SetSize, size},
};
#[allow(unused_imports)]
use log::*;

pub fn setup_terminal(mut stdout: &Stdout) -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;
    stdout.execute(SetSize(WIDTH as u16, HEIGHT as u16))?;

    Ok(())
}

pub fn set_styles(mut stdout: &Stdout) -> Result<(), Box<dyn std::error::Error>> {
    // stdout.execute(SetBackgroundColor(Color::Yellow))?;
    // stdout.execute(SetForegroundColor(Color::Red))?;
    // stdout.execute(Clear(terminal::ClearType::All))?;
    Ok(())
}

pub fn cleanup_terminal(
    mut stdout: &Stdout,
    original_size: (u16, u16),
) -> Result<(), Box<dyn std::error::Error>> {
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    stdout.execute(SetSize(original_size.0, original_size.1))?;
    terminal::disable_raw_mode()?;

    Ok(())
}

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
