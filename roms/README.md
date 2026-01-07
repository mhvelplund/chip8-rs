# Roms

* `blank.rom`: all zeroes, except for `0xE9E` which contains `0x12`. Combined with the following `0x00`, it becomes an
  instruction that jumps to '0x200' causing and endless loop.