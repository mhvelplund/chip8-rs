//! State of the CHIP-8 interpreter
//! This module defines the `State` struct which holds the entire state of the CHIP-8 interpreter,
//! including memory, registers, timers, stack, and display.
//!
//! The memory layout is as follows:
//! - 0x000 to 0x1FF: Reserved for the interpreter (including font set)
//! - 0x200 to 0xFFF: Program memory and data
//! - 0xEA0 to 0xEFF: Call stack (not explicitly modeled in this implementation)
//! - 0xF00 to 0xFFF: Display refresh area (not explicitly modeled in this implementation)
//!
//! We don't actually model the stack, to keep things simple. In reality, the stack is an area of memory used to store up
//! to 8 12 bit addresses, but we just keep those addresses in an array growing from index 0. The area of memory is unused.
//!
//! The `State` struct provides methods to initialize the state, load a ROM into memory,
//! and bootstrap the built-in character set.
use crate::constants;
use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub struct State {
    pub screen: [bool; constants::WIDTH * constants::HEIGHT],

    pub delay_timer: u8,
    pub sound_timer: u8,

    /// Address register, only lower 12 bits used
    pub i: usize,

    /// Pixels are stored in order, left to right from the upper-left corner. True means on, false means off.
    pub memory: [u8; constants::MEMORY_SIZE],

    /// Program counter, only lower 12 bits used
    pub pc: usize,

    /// Up to 12 levels of nested return addresses
    pub stack: VecDeque<usize>,

    /// Registers V0 to VF. VF is the carry flag, while in subtraction, it is the "no borrow" flag. In the draw instruction VF is set upon pixel collision.
    pub v: [u8; 16],

    /// Currently pressed key, if any.
    pub key_pressed: Option<u8>,

    /// Time when the key was pressed.
    pub key_pressed_at: std::time::SystemTime,

    /// If the interpreter is waiting for a key press this will be some, and the value is the register index to store the key in.
    pub waiting_for_keypress: Option<usize>,
}

impl State {
    pub fn new() -> Self {
        let mut state = Self {
            delay_timer: 0,
            sound_timer: 0,
            i: 0,
            memory: [0; constants::MEMORY_SIZE],
            pc: 0x200,
            screen: [false; constants::WIDTH * constants::HEIGHT],
            stack: VecDeque::new(),
            v: [0; 16],
            key_pressed: None,
            key_pressed_at: std::time::SystemTime::now(),
            waiting_for_keypress: None,
        };
        state.bootstrap_character_rom();
        for i in (0x040..0x200).step_by(2) {
            // Insert a HALT instruction in unused memory to prevent accidental execution
            state.memory[i] = 0xFF;
            state.memory[i + 1] = 0xFF;
        }
        state.memory[0xE9E] = 0x12; // Insert a jump to start of program at 0x200 to prevent accidental execution of uninitialized memory
        for i in (0xEA0..=0xFFF).step_by(2) {
            // Insert a HALT instruction in unused memory to prevent accidental execution
            state.memory[i] = 0xFF;
            state.memory[i + 1] = 0xFF;
        }
        state
    }

    /// Load the built-in character set into memory in the ROM into memory in the first 512 bytes.
    /// Each character is 5 bytes (5 rows of 8 pixels, only the upper 4 bits are used).
    pub fn bootstrap_character_rom(&mut self) {
        let charmap: [[u8; 5]; 16] = [
            [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
            [0x20, 0x60, 0x20, 0x20, 0x70], // 1
            [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
            [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
            [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
            [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
            [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
            [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
            [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
            [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
            [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
            [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
            [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
            [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
            [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
            [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
        ];

        let mut i = 0;
        for char_bytes in &charmap {
            for &b in char_bytes {
                self.memory[constants::CHARACTER_SPRITE_OFFSET + i] = b;
                i += 1;
            }
        }
    }
}

impl TryFrom<&PathBuf> for State {
    type Error = std::io::Error;

    fn try_from(rom_path: &PathBuf) -> Result<Self, std::io::Error> {
        let mut state = State::new();

        let mut f = File::open(rom_path)?;
        let mut buffer: [u8; 4096] = [0; constants::MEMORY_SIZE];
        let n = f.read(&mut buffer)?;

        // Load the ROM into memory starting at address 0x200
        state.memory[0x200..n].copy_from_slice(&buffer[0x200..n]);

        Ok(state)
    }
}
