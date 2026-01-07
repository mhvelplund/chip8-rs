//! Constants used throughout the CHIP-8 emulator.
//!
//! Memory size, screen dimensions, character sprite offsets, and clock frequency are defined here.

/// Character sprites start at 0x000
pub const CHARACTER_SPRITE_OFFSET: usize = 0x000;

/// 48kHz
pub const CLOCK_FREQ: u32 = 48000;

pub const HEIGHT: usize = 32;

/// 4KB
pub const MEMORY_SIZE: usize = 4096;

pub const WIDTH: usize = 64;
