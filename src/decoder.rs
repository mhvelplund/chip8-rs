//! CHIP-8 Instruction Decoder and Executor
//!
//! This module provides functionality to decode and execute CHIP-8 instructions.
//! The main function `decode_and_execute` takes the current state of the interpreter,
//! decodes the instruction at the program counter, and modifies the state accordingly.

use crate::constants;
use crate::state;
use log::*;

/// Draw a sprite at position `x`, `y` with `N` bytes of sprite data starting at the address stored in `state.i`.
/// Set `VF` to `1` if any set pixels are changed to unset, and `0` otherwise.
///
/// # Arguments
/// * `state` - The current state of the CHIP-8 interpreter.
/// * `x` - The x coordinate to draw the sprite at.
/// * `y` - The y coordinate to draw the sprite at.
/// * `n` - The number of bytes of sprite data to draw.
fn draw_sprite(state: &mut state::State, x: usize, y: usize, n: usize) {
    todo!()
}

pub fn decode_and_execute(
    state: &mut state::State,
) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    let instruction: u16 =
        ((state.memory[state.pc] as u16) << 8) | (state.memory[state.pc + 1] as u16);

    state.pc += 2;
    state.pc &= 0xFFF;

    // See: https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Instruction-Set
    match instruction & 0xF000 {
        0x0000 => match instruction & 0x0FFF {
            0x0000 => {
                // 0x0000: No operation (NB: Not part of the original CHIP-8 instruction set)
            }
            0x00E0 => {
                // 0x00E0: Clear the display
                state.screen = [false; constants::WIDTH * constants::HEIGHT];
            }
            0x00EE => {
                // 0x00EE: Return from subroutine
                state.pc = state.stack.pop_back().ok_or("Stack underflow on RET")?;
            }
            _ => {
                // 0x0NNN: Execute machine language subroutine at address NNN
                warn!("Ignored instruction: {:04X}", instruction);
            }
        },
        0x1000 => {
            // 0x1NNN: Jump to address NNN
            let nnn = (instruction & 0x0FFF) as usize;
            state.pc = nnn;
        }
        0x2000 => {
            // 0x2NNN: Execute subroutine starting at address NNN

            //// No need for this limitation in our implementation.
            // if state.stack.len() >= 12 {
            //     return Err("Stack overflow on CALL".into());
            // }

            let nnn = (instruction & 0x0FFF) as usize;
            state.stack.push_back(state.pc);
            state.pc = nnn;
        }
        0x3000 => {
            // 0x3XNN: Skip the following instruction if the value of register VX equals NN
            let x = ((instruction & 0x0F00) >> 8) as usize;
            let nn = (instruction & 0x00FF) as u8;

            if state.v[x] == nn {
                state.pc += 2;
            }
        }
        0x4000 => {
            // 0x4XNN: Skip the following instruction if the value of register VX does not equal NN
            let x = ((instruction & 0x0F00) >> 8) as usize;
            let nn = (instruction & 0x00FF) as u8;
            if state.v[x] != nn {
                state.pc += 2;
            }
        }
        0x5000 => {
            // 0x5XY0: Skip the following instruction if the value of register VX is equal to the value of register VY
            let x = ((instruction & 0x0F00) >> 8) as usize;
            let y = ((instruction & 0x00F0) >> 4) as usize;
            if state.v[x] == state.v[y] {
                state.pc += 2;
            }
        }
        0x6000 => {
            // 0x6XNN: Store number NN in register VX
            let x = ((instruction & 0x0F00) >> 8) as usize;
            let nn = (instruction & 0x00FF) as u8;
            state.v[x] = nn;
        }
        0x7000 => {
            // 0x7XNN: Add the value NN to register VX (no carry flag)
            let x = ((instruction & 0x0F00) >> 8) as usize;
            let nn = (instruction & 0x00FF) as u8;
            state.v[x] = state.v[x].wrapping_add(nn);
        }
        0x8000 => match instruction & 0x000F {
            0x0 => {
                // 0x8XY0: Store the value of register VY in register VX
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                state.v[x] = state.v[y];
            }
            0x1 => {
                // 0x8XY1: Set VX to VX OR VY
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                state.v[x] |= state.v[y];
            }
            0x2 => {
                // 0x8XY2: Set VX to VX AND VY
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                state.v[x] &= state.v[y];
            }
            0x3 => {
                // 0x8XY3: Set VX to VX XOR VY
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                state.v[x] ^= state.v[y];
            }
            0x4 => {
                // 0x8XY4: Add the value of register VY to register VX (set carry flag)
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                let (result, did_overflow) = state.v[x].overflowing_add(state.v[y]);
                state.v[x] = result;
                state.v[0xF] = if did_overflow { 1 } else { 0 };
            }
            0x5 => {
                // 0x8XY5: Subtract the value of register VY from register VX (set borrow flag)
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                let (result, did_overflow) = state.v[x].overflowing_sub(state.v[y]);
                state.v[x] = result;
                state.v[0xF] = if did_overflow { 0 } else { 1 };
            }
            0x6 => {
                // 0x8XY6: Store the value of register VY shifted right one bit in register VX
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                state.v[0xF] = state.v[y] & 0b0000_0001;
                state.v[x] = state.v[y] >> 1;
            }
            0x7 => {
                // 0x8XY7: Set register VX to the value of VY minus VX (set borrow flag)
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                let (result, did_overflow) = state.v[y].overflowing_sub(state.v[x]);
                state.v[x] = result;
                state.v[0xF] = if did_overflow { 0 } else { 1 };
            }
            0xE => {
                // 0x8XYE: Store the value of register VY shifted left one bit in register VX
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                state.v[0xF] = (state.v[y] & 0b1000_0000) >> 7;
                state.v[x] = state.v[y] << 1;
            }
            _ => {
                unknown_op(instruction);
            }
        },
        0x9000 => {
            // 0x9XY0: Skip the following instruction if the value of register VX is not equal to the value of register VY
            let x = ((instruction & 0x0F00) >> 8) as usize;
            let y = ((instruction & 0x00F0) >> 4) as usize;

            match instruction & 0x000F {
                0x0 => {
                    if state.v[x] != state.v[y] {
                        state.pc += 2;
                    }
                }
                _ => {
                    unknown_op(instruction);
                }
            }
        }
        0xA000 => {
            // 0xANNN: Store memory address NNN in register I
            let nnn = (instruction & 0x0FFF) as usize;
            state.i = nnn;
        }
        0xB000 => {
            // 0xBNNN: Jump to address NNN plus V0
            let nnn = (instruction & 0x0FFF) as usize;
            state.pc = nnn + (state.v[0] as usize);
        }
        0xC000 => {
            // 0xCXNN: Set VX to a random number with a mask of NN
            let x = ((instruction & 0x0F00) >> 8) as usize;
            let nn = (instruction & 0x00FF) as u8;

            let rand_byte: u8 =
                ((state.pc + state.i + state.v.iter().sum::<u8>() as usize) & 0xFF) as u8; // FIXME: Placeholder for random byte generation
            state.v[x] = rand_byte & nn;
        }
        0xD000 => {
            // 0xDXYN: Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I.
            // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
            let x = ((instruction & 0x0F00) >> 8) as usize;
            let y = ((instruction & 0x00F0) >> 4) as usize;
            let n = (instruction & 0x000F) as usize;
            draw_sprite(state, x, y, n);
        }
        0xE000 => {
            let x = ((instruction & 0x0F00) >> 8) as usize;
            match instruction & 0x00FF {
                0x9E => {
                    // 0xEX9E: Skip the following instruction if the key stored in VX is pressed
                    if state.key_pressed == Some(state.v[x]) {
                        state.pc += 2;
                    }
                    state.key_pressed = None;
                }
                0xA1 => {
                    // 0xEXA1: Skip the following instruction if the key stored in VX is not pressed
                    if state.key_pressed != Some(state.v[x]) {
                        state.pc += 2;
                    }
                    state.key_pressed = None;
                }
                _ => {
                    unknown_op(instruction);
                }
            }
        }
        0xF000 => {
            let x = ((instruction & 0x0F00) >> 8) as usize;
            match instruction & 0x00FF {
                0x07 => {
                    // 0xFX07: Store the current value of the delay timer in register VX
                    state.v[x] = state.delay_timer;
                }
                0x0A => {
                    // 0xFX0A: Wait for a key press and store the value of the key in register VX
                    state.waiting_for_keypress = Some(x);
                }
                0x15 => {
                    // 0xFX15: Set the delay timer to the value of register VX
                    state.delay_timer = state.v[x];
                }
                0x18 => {
                    // 0xFX18: Set the sound timer to the value of register VX
                    state.sound_timer = state.v[x];
                }
                0x1E => {
                    // 0xFX1E: Add the value stored in register VX to register I
                    state.i = state.i.wrapping_add(state.v[x] as usize) & 0xFFF;
                }
                0x29 => {
                    // 0xFX29: Set I to the location of the sprite for the character in VX.
                    // Characters 0-F (in hexadecimal) are represented by a 4x5 font
                    state.i =
                        constants::CHARACTER_SPRITE_OFFSET + ((state.v[x] & 0xF) as usize) * 5;
                }
                0x33 => {
                    // 0xFX33: Store the binary-coded decimal representation of VX,
                    // with the hundreds digit at the address in I, the tens digit at I+1, and the ones digit at I+2
                    let (hundreds, tens, ones) = bcd(state.v[x]);
                    state.memory[state.i] = hundreds;
                    state.memory[state.i + 1] = tens;
                    state.memory[state.i + 2] = ones;
                }
                0x55 => {
                    // 0xFX55: Store registers V0 through VX in memory starting at location I
                    for i in 0..=x {
                        state.memory[state.i + i] = state.v[i];
                        state.i += x + 1;
                    }
                }
                0x65 => {
                    // 0xFX65: Read registers V0 through VX from memory starting at location I
                    for i in 0..=x {
                        state.v[i] = state.memory[state.i + i];
                        state.i += x + 1;
                    }
                }
                0xFF => {
                    // 0xFXFF: Halt execution (NB: Not part of the original CHIP-8 instruction set)
                    return Ok(Some(x));
                }
                _ => {
                    unknown_op(instruction);
                }
            }
        }
        _ => {
            unknown_op(instruction);
        }
    }

    Ok(None)
}

/// Convert a value to its binary-coded decimal (BCD) representation.
///
/// # Arguments
/// * `value` - The value to convert to BCD.
///
/// # Returns
/// A tuple containing the hundreds, tens, and ones digits of the BCD representation.
fn bcd(value: u8) -> (u8, u8, u8) {
    let hundreds = value / 100;
    let tens = (value % 100) / 10;
    let ones = value % 10;
    (hundreds, tens, ones)
}

pub fn unknown_op(instruction: u16) {
    warn!("Ignored instruction: {instruction:04X}");
}
