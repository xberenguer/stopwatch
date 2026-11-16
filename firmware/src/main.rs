#![no_std]
#![no_main]
mod panel;

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    dma_rx_buffer, dma_tx_buffer,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config as I2cConfig, I2c},
    psram::{Psram, PsramConfig, PsramMode, PsramSize, SpiRamFreq},
    spi::{
        self,
        master::{Config as SpiConfig, Spi},
    },
    time::{Instant, Rate},
};
use stopwatch::{
    Button, Debouncer, Stopwatch,
    render::{PIXELS, Renderer, Rows},
};
esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("Stopwatch panic: {}", info);
    loop {
        core::hint::spin_loop();
    }
}
fn micros() -> u64 {
    Instant::now().duration_since_epoch().as_micros()
}

struct Inputs {
    start: Input<'static>,
    lap: Input<'static>,
    start_debounce: Debouncer,
    lap_debounce: Debouncer,
    pending: [Option<(Button, u64)>; 2],
}
impl Inputs {
    fn poll(&mut self) {
        let now = micros();
        if let Some(t) = self.start_debounce.poll(self.start.is_high(), now) {
            self.pending[0] = Some((Button::StartPause, t));
        }
        if let Some(t) = self.lap_debounce.poll(self.lap.is_low(), now) {
            self.pending[1] = Some((Button::LapReset, t));
        }
    }
    fn apply(&mut self, watch: &mut Stopwatch) {
        // Timestamp order is preserved when both edges arrive during a transfer.
        let a = self.pending[0].take();
        let b = self.pending[1].take();
        let events = match (a, b) {
            (Some(a), Some(b)) if a.1 > b.1 => [Some(b), Some(a)],
            _ => [a, b],
        };
        for (button, time) in events.into_iter().flatten() {
            watch.press(button, time);
        }
    }
}

#[esp_hal::main]
fn main() -> ! {
    let p = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    // Buttons only. Touch stays in reset and the speaker amplifier stays off.
    let _touch_reset = Output::new(p.GPIO2, Level::Low, OutputConfig::default());
    let _amp = Output::new(p.GPIO46, Level::Low, OutputConfig::default());
    let mut inputs = Inputs {
        start: Input::new(p.GPIO3, InputConfig::default()),
        lap: Input::new(p.GPIO0, InputConfig::default().with_pull(Pull::Up)),
        start_debounce: Debouncer::default(),
        lap_debounce: Debouncer::default(),
        pending: [None; 2],
    };
    inputs.poll();
    // Preserve factory PMIC rails and charging. Quiet unused IMU and microphones.
    let mut i2c = I2c::new(p.I2C0, I2cConfig::default().with_frequency(Rate::from_khz(400)))
        .unwrap()
        .with_sda(p.GPIO15)
        .with_scl(p.GPIO14);
    let mut b = [0u8];
    if i2c.write_read(0x6bu8, &[0], &mut b).is_ok() && b[0] == 5 {
        if i2c.write_read(0x6bu8, &[8], &mut b).is_ok() {
            let _ = i2c.write(0x6bu8, &[8, b[0] & !3]);
        }
        if i2c.write_read(0x6bu8, &[2], &mut b).is_ok() {
            let _ = i2c.write(0x6bu8, &[2, b[0] | 2]);
        }
    }
    if i2c.write_read(0x40u8, &[1], &mut b).is_ok() {
        for r in 0x47..=0x4c {
            let _ = i2c.write(0x40u8, &[r, 255]);
        }
        let _ = i2c.write(0x40u8, &[1, 0x7f]);
        let _ = i2c.write(0x40u8, &[6, 7]);
    }
    let psram = Psram::new(
        p.PSRAM,
        PsramConfig {
            mode: PsramMode::OctalSpi,
            size: PsramSize::Size(8 * 1024 * 1024),
            ram_frequency: SpiRamFreq::Freq80m,
            ..PsramConfig::default()
        },
    );
    let (ptr, len) = psram.raw_parts();
    assert!(len >= PIXELS * 2 && ptr.align_offset(2) == 0);
    // SAFETY: the exclusively owned, aligned PSRAM lives for the infinite main
    // loop. No allocator or other user accesses this framebuffer region.
    let fb = unsafe { core::slice::from_raw_parts_mut(ptr.cast::<u16>(), PIXELS) };
    fb.fill(0);
    let spi = Spi::new(
        p.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(40)).with_mode(spi::Mode::_0),
    )
    .unwrap()
    .with_sck(p.GPIO38)
    .with_sio0(p.GPIO4)
    .with_sio1(p.GPIO5)
    .with_sio2(p.GPIO6)
    .with_sio3(p.GPIO7)
    .with_cs(p.GPIO12)
    .with_dma(p.DMA_CH0)
    .with_buffers(dma_rx_buffer!(4).unwrap(), dma_tx_buffer!(64).unwrap());
    let mut reset = Output::new(p.GPIO1, Level::High, OutputConfig::default());
    let mut panel = panel::Panel::new(spi, dma_tx_buffer!(16000).unwrap(), &mut reset);
    let mut watch = Stopwatch::default();
    let mut renderer = Renderer::default();
    let mut rows = Rows::default();
    rows.full();
    renderer.draw(fb, &mut rows, &watch, micros(), || inputs.poll());
    panel.flush(fb, &rows, || inputs.poll());
    rows.clear();
    panel.show();
    let delay = Delay::new();
    let mut last_frame = 0;
    loop {
        inputs.poll();
        inputs.apply(&mut watch);
        let now = micros();
        watch.tick(now);
        if now - last_frame >= 20_000 {
            renderer.draw(fb, &mut rows, &watch, now, || inputs.poll());
            panel.flush(fb, &rows, || inputs.poll());
            rows.clear();
            last_frame = now;
        }
        delay.delay_micros(200);
    }
}
