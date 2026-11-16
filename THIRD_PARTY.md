# Third-party material

- `esp-hal` 1.2.2 and `esp-println` 0.17.0: MIT OR Apache-2.0. `esp-bootloader-esp-idf` 0.6.0: MIT OR Apache-2.0. See the pinned dependency sources for their license notices and those of transitive dependencies.
- The CO5300 driver and partition-aware installer are adapted from the local Roulette Round Rust 0.3.0 project for this same board. Display initialization and rotation were checked against the local Seeder Fork board implementation and Waveshare examples.
- Text masks are rasterized from the locally installed Helvetica Neue Light system font. The font file is not included. Generated masks are in `assets/`; the generating script records the typeface, size and face index. The MIT license covers project code, not third-party typeface rights.
- Hardware documentation and examples: https://github.com/waveshareteam/ESP32-S3-Touch-AMOLED-1.75C and https://docs.waveshare.com/ESP32-S3-Touch-AMOLED-1.75C.
