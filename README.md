# Stopwatch

A minimal, button-only Rust stopwatch for the Waveshare ESP32-S3-Touch-AMOLED-1.75C.

[Flash over USB](https://xn--sg8hhb.ws/flasher/?project=stopwatch) · [Get the device](https://www.waveshare.com/esp32-s3-touch-amoled-1.75c.htm) · [Firmware](https://github.com/xberenguer/stopwatch/releases/latest)

| Ready | Running | Lap | Hours |
|---|---|---|---|
| <img src="previews/ready-charcoal.png" width="180" alt="Ready stopwatch on a black circular display"> | <img src="previews/running-total.gif" width="180" alt="Running stopwatch animation showing only the total time"> | <img src="previews/lap-charcoal.png" width="180" alt="Stopwatch showing a recorded lap below the total"> | <img src="previews/hours-charcoal.png" width="180" alt="Stopwatch showing 12:34:56.78 with hour digits"> |

Hold the device with both buttons on top.

| Button | Running | Paused |
|---|---|---|
| Large, top left | Pause | Start / resume |
| Small, top right | Lap | Reset |

When stopped at zero, hold the small button for 0.6 seconds to turn the screen off. Keep holding for 1.2 seconds total to see the battery percentage until you release. Releasing after either threshold puts the stopwatch to sleep. A small-button press wakes the screen. A large-button press wakes it and starts timing.

On battery power, turning the screen off enters ESP32 light sleep. If USB power is connected when the screen turns off, the USB flasher remains available.

Laps are immediate. Only the latest lap is kept.

## Build

Firmware needs the Espressif Xtensa Rust toolchain, an ESP32-S3 GCC linker, and esptool 4.9.0.

```sh
cargo +stable test --locked
python3 tools/build.py
```
