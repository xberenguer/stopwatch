# Stopwatch

A plain, button-only stopwatch for **Waveshare ESP32-S3-Touch-AMOLED-1.75C**. Native Rust, black AMOLED, centered Helvetica Neue Light numerals. No menus, touchscreen, wireless, sound, or splash screen.

The compact revision has **590 lines of Rust** across the complete implementation, excluding tests: 128 for timing/buttons/formatting, 167 for rendering, 30 for reading packed font data, and 265 for board setup and the display driver. Font pixels and metrics are binary assets. The application image is 162,000 bytes, down from 181,152 bytes. The same 11 tests pass, and all five preview states are pixel-identical to the preceding working version.

Hold it in the same orientation as Seeder Fork: **270° clockwise from buttons-right**, with both buttons on top. Large PWR is top left; small BOOT is top right. The framebuffer uses the exact equivalent of Seeder's canvas rotation 1.

| Button | While running | While paused |
|---|---|---|
| Large PWR, top left | Pause | Start / resume |
| Small BOOT, top right | Record lap | Reset |

The large time always shows the total. Recording a lap adds the latest lap interval beneath it. Only the latest interval is retained; there is no lap history. Starting again preserves elapsed time and the latest lap. Reset clears both. Hundredths of a second are shown; after one hour the display automatically includes hours, up to 99:59:59.99, where it stops. The board's hardware long-press power-off remains available. Power-off/restart clears the stopwatch.

[Project page](https://xn--sg8hhb.ws/stopwatch/) · [Browser installer](https://xn--sg8hhb.ws/flasher/?project=stopwatch)

## Install

`bin/stopwatch-app.bin` is an **application-only image** for the original seven-partition 1.75C factory layout, with the application at **0x200000**. Use the supplied installer, which checks the full layout and board identity, backs up overwritten sectors, verifies the write, and attempts rollback if installation fails.

Install Python 3 and esptool 4.9.0 in a virtual environment:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install esptool==4.9.0
.venv/bin/python -m esptool --chip esp32s3 --port /dev/cu.usbmodemYOUR_PORT flash_id
.venv/bin/python tools/flash.py \
  --port /dev/cu.usbmodemYOUR_PORT \
  --expect-mac YOUR_BOARD_MAC \
  --application bin/stopwatch-app.bin \
  --backup-dir ./private-backups
```

The installer refuses other flash layouts and board identities. Backups may contain the previous application's private data; keep them locally. This build targets the **1.75C**, whose GPIO3 is the large button; the non-C 1.75 is different.

## Build

Host checks and previews use stable Rust, with no external crates:

```sh
cargo +stable test --locked
cargo +stable run --release --example preview -- previews
```

Firmware requires the Espressif Xtensa Rust toolchain installed by `espup`, `rust-src`, and an ESP32-S3 GCC linker. After installing those, from this directory:

```sh
.venv/bin/python tools/build.py
```

The build helper finds `xtensa-esp32s3-elf-gcc` or modern `xtensa-esp-elf-gcc` on PATH, or the PlatformIO ESP32-S3 toolchain. Use `--gcc /path/to/compiler` and `--toolchain NAME` for alternate installations. Both Cargo lockfiles are included. The generated font masks are included; Pillow and the system font are needed only to regenerate them with `tools/make_font.py`.

## Implementation and validation

`src/lib.rs` contains the stopwatch and debounced controls. Timing uses the ESP32-S3 monotonic microsecond hardware clock; rendering runs separately at up to 50 frames per second. Input is polled between frames and throughout display DMA transfers. Presses use their first observed stable-edge timestamp, so debounce does not add 12 ms to measured intervals. Holds do not repeat. Keys held during startup must be released before activation.

`src/render.rs` is shared by the physical display and host previews. `firmware/src/panel.rs` uses the existing 1.75C CO5300 initialization, 40 MHz QSPI, native x offset 6, and 128/255 brightness. Partial-update windows expand to even pixel boundaries, as required by the panel. Touch is held in hardware reset, the amplifier is off, and unused IMU/microphones are put to sleep. PMIC rails and charging settings are preserved.

All 11 host tests pass, covering pause/resume, lap intervals, reset, time formatting, maximum duration, button bounce/holds/startup, frame updates, text bounds, the Seeder rotation, and panel-window alignment. Clippy passes with warnings denied. Firmware was cross-compiled and its image validated with esptool. Preview PNGs come from the same Rust renderer.

The preceding display-corrected firmware was installed on the user's 32 MB Waveshare on November 16, 2026. Its application was read-back verified, with the original overwritten sectors backed up privately and other partitions preserved. The user confirmed the physical screen's text is clean after starting and pausing. The compact revision was subsequently installed and read-back verified on the same Waveshare. Its physical appearance has not been separately confirmed after that refactor. Physical lap/reset and battery-runtime checks were not separately recorded. Clock accuracy follows the board oscillator and has not been calibrated against a reference.

Hardware reference: [Waveshare 1.75C documentation](https://docs.waveshare.com/ESP32-S3-Touch-AMOLED-1.75C).

