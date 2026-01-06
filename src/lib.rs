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

        if decode_and_execute(&mut state)? {
            // Halt execution
            break;
        }

        // TODO: Update timers at 60Hz

        // TODO: Handle input

        // TODO: Render display

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

    // See: https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Instruction-Set
    match instruction & 0xF000 {
        0x0000 => match instruction & 0x0FFF {
            0x00E0 => {
                // 0x00E0: Clear the display
                state.screen = [false; WIDTH * HEIGHT];
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

            match (instruction & 0x000F) {
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
            todo!();
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
                    todo!()
                }
                0xA1 => {
                    // 0xEXA1: Skip the following instruction if the key stored in VX is not pressed
                    todo!()
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
                0x15 => {
                    // 0xFX15: Set the delay timer to the value of register VX
                    state.delay_timer = state.v[x];
                }
                0x1E => {
                    // 0xFX1E: Add the value stored in register VX to register I
                    state.i = state.i.wrapping_add(state.v[x] as usize) & 0xFFF;
                }
                0x29 => {
                    // 0xFX29: Set I to the location of the sprite for the character in VX.
                    // Characters 0-F (in hexadecimal) are represented by a 4x5 font
                    todo!()
                }
                0x33 => {
                    // 0xFX33: Store the binary-coded decimal representation of VX,
                    // with the hundreds digit at the address in I, the tens digit at I+1, and the ones digit at I+2
                    todo!()
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
                _ => {
                    unknown_op(instruction);
                }
            }
        }
        _ => {
            unknown_op(instruction);
        }
    }

    Ok(false)
}

fn draw_sprite(state: &mut State, x: usize, y: usize, n: usize) {
    // Draw a sprite at position x, y with N bytes of sprite data starting at the address stored in state.i
    // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
    todo!()
}

fn unknown_op(instruction: u16) {
    warn!("Ignored instruction: {instruction:04X}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_clear_screen() {
        let mut state = State::new();

        state.screen[0] = true; // Set a pixel
        state.screen[WIDTH * HEIGHT - 1] = true; // Set another pixel

        // 0x00E0: Clear the display
        state.memory[0x200] = 0x00;
        state.memory[0x201] = 0xE0;

        decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.screen, [false; WIDTH * HEIGHT]);
        assert_eq!(state.pc, 0x202);
    }

    #[test]
    fn instruction_jump() {
        let mut state = State::new();
        // 0x1NNN: Jump to address NNN
        state.memory[0x200] = 0x12;
        state.memory[0x201] = 0x34;

        decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x234);
    }

    #[test]
    fn instruction_call_and_return() {
        let mut state = State::new();

        // 0x2NNN: Execute subroutine starting at address NNN
        state.memory[0x200] = 0x23; // CALL 0x345
        state.memory[0x201] = 0x45; // CALL 0x345

        // 0x00EE: Return from subroutine
        state.memory[0x345] = 0x00; // RET instruction high byte
        state.memory[0x346] = 0xEE; // RET instruction low byte

        decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x345);
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], 0x202);

        decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x202);
        assert_eq!(state.stack.len(), 0);
    }

    #[test]
    fn instruction_call_stack_underflow() {
        let mut state = State::new();

        // 0x00EE: Return from subroutine before any CALL to cause stack underflow
        state.memory[0x200] = 0x00; // RET instruction high byte
        state.memory[0x201] = 0xEE; // RET instruction low byte

        decode_and_execute(&mut state).expect_err("Should have caused a stack underflow");
    }

    #[test]
    fn instruction_skip_if_equal() {
        let mut state = State::new();
        // 0x3XNN: Skip the following instruction if the value of register VX equals NN
        state.v[0] = 0x42;
        state.memory[0x200] = 0x30; // SE V0, 0x42
        state.memory[0x201] = 0x42; // SE V0, 0x42

        decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x204); // Should have skipped the next instruction
    }

    #[test]
    fn instruction_no_skip_if_not_equal() {
        let mut state = State::new();
        // 0x3XNN: Skip the following instruction if the value of register VX equals NN
        state.v[0] = 0x41;
        state.memory[0x200] = 0x30; // SE V0, 0x42
        state.memory[0x201] = 0x42; // SE V0, 0x42

        decode_and_execute(&mut state).expect("Failed to execute instruction");

        assert_eq!(state.pc, 0x202); // Should not have skipped the next instruction
    }
}
