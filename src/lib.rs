#![allow(unused)]

use log::*;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, SystemTimeError};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const MEMORY_SIZE: usize = 4096; // 4KB

const CLOCK_FREQ: u32 = 48000; // 48kHz

struct State {
    screen: [bool; WIDTH * HEIGHT],

    // Memory layout:
    // 256 bytes for display refresh (0xF00-0xFFF)
    // 96 bytes for call stack       (0xEA0-0xEFF) (See below)
    // 3744 bytes for program memory (0x000-0xE9F)
    //
    // We don't actually model the stack, to keep things simple. In reality, the stack is an area of memory used to store up
    // to 8 12 bit addresses, but we just keep those addresses in an array growing from index 0. The area of memory is unused.
    //
    delay_timer: u8,
    i: usize,                  // Address register, only lower 12 bits used
    memory: [u8; MEMORY_SIZE], // Pixels are stored in order, left to right from the upper-left corner. True means on, false means off.
    pc: usize,                 // Program counter, only lower 12 bits used
    stack: VecDeque<usize>,    // Up to 12 levels of nested return addresses
    v: [u8; 16], // Registers V0 to VF. VF is the carry flag, while in subtraction, it is the "no borrow" flag. In the draw instruction VF is set upon pixel collision.
}

impl State {
    pub fn new() -> Self {
        Self {
            delay_timer: 0,
            i: 0,
            memory: [0; MEMORY_SIZE],
            pc: 0x200,
            screen: [false; WIDTH * HEIGHT],
            stack: VecDeque::new(),
            v: [0; 16],
        }
    }
}

pub fn run_rom(rom_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = State::new();
    let tick_length = Duration::from_secs(1) / CLOCK_FREQ;

    loop {
        let tick_start = SystemTime::now();

        let stop = decode_and_execute(&mut state)?;

        if stop {
            break;
        }

        // Wait for tick
        let elapsed = tick_start.elapsed().unwrap_or(Duration::from_secs(0));
        if elapsed < tick_length {
            std::thread::sleep(tick_length - elapsed);
        }
    }
    Ok(())
}

fn decode_and_execute(state: &mut State) -> Result<bool, Box<dyn std::error::Error>> {
    let instruction: u16 =
        ((state.memory[state.pc] as u16) << 8) | (state.memory[state.pc + 1] as u16);

    state.pc += 2;
    state.pc &= 0xFFF;

    match instruction & 0xF000 {
        0x0000 => match instruction & 0x0FFF {
            0x00E0 => {
                // Clear the display
                state.screen = [false; WIDTH * HEIGHT];
            }
            0x00EE => {
                // Return from subroutine
                state.pc = state.stack.pop_back().ok_or("Stack underflow on RET")?;
            }
            _ => {
                // Execute machine language subroutine at address NNN
                warn!("Ignored instruction: {:04X}", instruction);
            }
        },
        0x1000 => {
            // Jump to address NNN
            state.pc = (instruction & 0x0FFF) as usize;
        }
        0x2000 => {
            // Call subroutine at NNN
            if state.stack.len() >= 12 {
                return Err("Stack overflow on CALL".into());
            }
            state.stack.push_back(state.pc);
            state.pc = (instruction & 0x0FFF) as usize;
        }
        _ => {
            warn!("Ignored instruction: {:04X}", instruction);
        }
    }

    Ok(false)
}
