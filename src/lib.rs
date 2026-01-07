#![allow(unused)]

use log::*;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

mod constants;
mod decoder;
mod state;

pub fn run_rom(rom_path: PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
    let mut state = state::State::try_from(&rom_path)?;

    let tick_length = Duration::from_secs(1) / constants::CLOCK_FREQ;

    let exit_code = loop {
        let tick_start = SystemTime::now();

        if let Some(exit_code) = decoder::decode_and_execute(&mut state)? {
            // Halt execution
            break exit_code;
        }

        // TODO: Update timers at 60Hz

        // TODO: Handle input

        // TODO: Render display

        // Wait for tick
        let elapsed = tick_start.elapsed().unwrap_or(Duration::from_secs(0));
        if elapsed < tick_length {
            std::thread::sleep(tick_length - elapsed);
        }
    };

    debug!("Program halted with exit code {}", exit_code);

    Ok(exit_code)
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
