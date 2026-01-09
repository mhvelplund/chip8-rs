#![allow(unused)]

use crate::term::{cleanup_terminal, set_styles, setup_terminal, should_exit};
use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode, KeyEvent, poll};
use crossterm::terminal::{Clear, ClearType, size};
use crossterm::{ExecutableCommand, execute};
use log::*;
use std::io::Write;
use std::io::stdout;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

mod constants;
mod decoder;
mod state;
mod term;

pub fn run_rom(rom_path: PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
    let mut state = state::State::try_from(&rom_path)?;

    let tick_length = Duration::from_secs(1) / constants::CLOCK_FREQ;

    let original_size = size()?;
    let mut stdout = stdout();

    setup_terminal()?;
    set_styles()?;

    let exit_code = loop {
        let tick_start: SystemTime = SystemTime::now();

        if state.waiting_for_keypress.is_none()
            && let Some(exit_code) = decoder::decode_and_execute(&mut state)?
        {
            // Halt execution
            break exit_code;
        }

        // TODO: Update timers at 60Hz

        if poll(Duration::from_millis(0))? {
            let event = event::read()?;

            // TODO: update keys down in state

            if should_exit(&event)? {
                break 0;
            }

            if let Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) = event
            {
                state.key_pressed_at = SystemTime::now();

                let key = match c {
                    '1' => Some(0x0),
                    '2' => Some(0x1),
                    '3' => Some(0x2),
                    '4' => Some(0x3),
                    'q' => Some(0x4),
                    'w' => Some(0x5),
                    'e' => Some(0x6),
                    'r' => Some(0x7),
                    'a' => Some(0x8),
                    's' => Some(0x9),
                    'd' => Some(0xA),
                    'f' => Some(0xB),
                    'z' => Some(0xC),
                    'x' => Some(0xD),
                    'c' => Some(0xE),
                    'v' => Some(0xF),
                    _ => None,
                };
                state.key_pressed = key;

                if let Some(reg) = state.waiting_for_keypress
                    && let Some(key) = key
                {
                    state.v[reg] = key;
                    state.waiting_for_keypress = None;
                }
            }

            execute!(stdout, MoveTo(0, (constants::HEIGHT + 1) as u16));
            execute!(stdout, Clear(ClearType::CurrentLine));
            // write!(stdout, "{event:?}");
            write!(stdout, "{:?}", state.key_pressed);
        }

        for row in 0..constants::HEIGHT {
            execute!(stdout, MoveTo(0, row as u16));

            for column in 0..constants::WIDTH {
                let pixel_on = state.screen[row * constants::WIDTH + column];
                let symbol = if pixel_on { '█' } else { ' ' };
                write!(stdout, "{}", symbol)?;
            }
        }

        execute!(stdout, MoveTo(0, constants::HEIGHT as u16));
        write!(stdout, "PC: {:03X}", state.pc);

        // Check for keypress timeout
        let elapsed = elapsed_time(&state.key_pressed_at);
        if elapsed > constants::KEY_PRESS_TIMEOUT_MS {
            state.key_pressed = None;
            execute!(stdout, MoveTo(0, (constants::HEIGHT + 1) as u16));
            execute!(stdout, Clear(ClearType::CurrentLine));
        }

        // Wait for tick
        let elapsed = elapsed_time(&tick_start);
        if elapsed < tick_length {
            std::thread::sleep(tick_length - elapsed);
        }
    };

    cleanup_terminal(original_size)?;

    debug!("Program halted with exit code {}", exit_code);

    Ok(exit_code)
}

/// Returns the elapsed time since the given SystemTime.
/// If the SystemTime is in the future, returns a Duration of zero.
///
/// # Arguments
/// * `t` - A reference to a SystemTime instance.
///
/// # Returns
/// A Duration representing the elapsed time since `t`.
fn elapsed_time(t: &SystemTime) -> Duration {
    t.elapsed().unwrap_or(Duration::from_secs(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_clear_screen() {
        let mut state = state::State::new();

        state.screen[0] = true; // Set a pixel
        state.screen[constants::WIDTH * constants::HEIGHT - 1] = true; // Set another pixel

        // 0x00E0: Clear the display
        state.memory[0x200] = 0x00;
        state.memory[0x201] = 0xE0;

        decoder::decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.screen, [false; constants::WIDTH * constants::HEIGHT]);
        assert_eq!(state.pc, 0x202);
    }

    #[test]
    fn instruction_jump() {
        let mut state = state::State::new();
        // 0x1NNN: Jump to address NNN
        state.memory[0x200] = 0x12;
        state.memory[0x201] = 0x34;

        decoder::decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x234);
    }

    #[test]
    fn instruction_call_and_return() {
        let mut state = state::State::new();

        // 0x2NNN: Execute subroutine starting at address NNN
        state.memory[0x200] = 0x23; // CALL 0x345
        state.memory[0x201] = 0x45; // CALL 0x345

        // 0x00EE: Return from subroutine
        state.memory[0x345] = 0x00; // RET instruction high byte
        state.memory[0x346] = 0xEE; // RET instruction low byte

        decoder::decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x345);
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], 0x202);

        decoder::decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x202);
        assert_eq!(state.stack.len(), 0);
    }

    #[test]
    fn instruction_call_stack_underflow() {
        let mut state = state::State::new();

        // 0x00EE: Return from subroutine before any CALL to cause stack underflow
        state.memory[0x200] = 0x00; // RET instruction high byte
        state.memory[0x201] = 0xEE; // RET instruction low byte

        decoder::decode_and_execute(&mut state).expect_err("Should have caused a stack underflow");
    }

    #[test]
    fn instruction_skip_if_equal() {
        let mut state = state::State::new();
        // 0x3XNN: Skip the following instruction if the value of register VX equals NN
        state.v[0] = 0x42;
        state.memory[0x200] = 0x30; // SE V0, 0x42
        state.memory[0x201] = 0x42; // SE V0, 0x42

        decoder::decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x204); // Should have skipped the next instruction
    }

    #[test]
    fn instruction_no_skip_if_not_equal() {
        let mut state = state::State::new();
        // 0x3XNN: Skip the following instruction if the value of register VX equals NN
        state.v[0] = 0x41;
        state.memory[0x200] = 0x30; // SE V0, 0x42
        state.memory[0x201] = 0x42; // SE V0, 0x42

        decoder::decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x202); // Should not have skipped the next instruction
    }
}
