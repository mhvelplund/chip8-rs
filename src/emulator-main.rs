use std::path::PathBuf;

use chip8_rs::run_rom;
use clap::Parser;

#[allow(unused_imports)]
use log::*;

#[derive(Parser, Debug)]
#[command(version, about="A CHIP-8 emulator.", long_about = None, author)]
struct Args {
    rom_path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let rom_path = args
        .rom_path
        .canonicalize()
        .map_err(|e| format!("ROM not found '{}': {}", args.rom_path.display(), e))?;

    let exit_code = run_rom(rom_path)?;
    info!("Program exited with code {}", exit_code);
    Ok(())
}
