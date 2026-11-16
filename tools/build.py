#!/usr/bin/env python3
"""Build native Rust firmware and an application-only ESP32-S3 image."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--toolchain', default='esp', help='espup Rust toolchain name')
p.add_argument('--gcc', help='ESP32-S3 GCC linker path')
p.add_argument('--offline', action='store_true')
a = p.parse_args()
root = Path(__file__).resolve().parents[1]
gcc = a.gcc or shutil.which('xtensa-esp32s3-elf-gcc') or shutil.which('xtensa-esp-elf-gcc')
if not gcc:
    candidate = Path.home() / '.platformio/packages/toolchain-xtensa-esp32s3/bin/xtensa-esp32s3-elf-gcc'
    if candidate.is_file(): gcc = str(candidate)
if not gcc:
    p.error('Install an ESP32-S3 GCC toolchain or supply --gcc /path/to/xtensa-esp32s3-elf-gcc')
env = os.environ.copy()
env['CARGO_TARGET_XTENSA_ESP32S3_NONE_ELF_LINKER'] = str(Path(gcc).resolve())
flags = ['-C', 'link-arg=-nostartfiles', '-C', 'link-arg=-Tlinkall.x']
if Path(gcc).name == 'xtensa-esp-elf-gcc':
    dyn = Path(gcc).resolve().parents[1] / 'lib/xtensa_esp32s3.so'
    if not dyn.is_file(): p.error('Generic Xtensa GCC needs lib/xtensa_esp32s3.so')
    flags += ['-C', f'link-arg=-mdynconfig={dyn}']
env['CARGO_ENCODED_RUSTFLAGS'] = '\x1f'.join(flags)
cmd = ['cargo', f'+{a.toolchain}', 'build', '--release', '--locked']
if a.offline: cmd.append('--offline')
subprocess.run(cmd, cwd=root / 'firmware', env=env, check=True)
elf = root / 'firmware/target/xtensa-esp32s3-none-elf/release/stopwatch-firmware'
out = root / 'bin'
out.mkdir(exist_ok=True)
shutil.copy2(elf, out / 'stopwatch.elf')
subprocess.run([sys.executable, '-m', 'esptool', '--chip', 'esp32s3', 'elf2image',
                '--flash_mode', 'dio', '--flash_freq', '80m', '--flash_size', '32MB',
                '-o', str(out / 'stopwatch-app.bin'), str(elf)], check=True)
print(f'Built {out / "stopwatch-app.bin"} (application only, factory offset 0x200000)')
