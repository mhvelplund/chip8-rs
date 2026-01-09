//! Constants used throughout the CHIP-8 emulator.
//!
//! Memory size, screen dimensions, character sprite offsets, and clock frequency are defined here.

use std::time::Duration;

/// Character sprites start at 0x000
pub const CHARACTER_SPRITE_OFFSET: usize = 0x000;

/// 48kHz
pub const CLOCK_FREQ: u32 = 48000;

/// Screen height in "pixels"
pub const HEIGHT: usize = 32;

/// 4KB
pub const MEMORY_SIZE: usize = 4096;

/// Screen width in "pixels"
pub const WIDTH: usize = 64;

/// Key presses time-out after 100 ms, if not polled. This is to handle our missing key-up events :/
pub const KEY_PRESS_TIMEOUT_MS: Duration = Duration::from_millis(100);
