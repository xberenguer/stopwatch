use esp_hal::{
    Blocking,
    delay::Delay,
    dma::DmaTxBuf,
    gpio::Output,
    spi::master::{Address, Command, DataMode, SpiDma},
};
use stopwatch::render::{Rows, W};
pub struct Panel {
    spi: Option<SpiDma<'static, Blocking>>,
    pixels: Option<DmaTxBuf>,
}
impl Panel {
    pub fn new(
        spi: SpiDma<'static, Blocking>,
        pixels: DmaTxBuf,
        reset: &mut Output<'static>,
    ) -> Self {
        let mut s = Self { spi: Some(spi), pixels: Some(pixels) };
        let delay = Delay::new();
        reset.set_high();
        delay.delay_millis(10);
        reset.set_low();
        delay.delay_millis(200);
        reset.set_high();
        delay.delay_millis(200);
        s.reg(0x11, &[]);
        delay.delay_millis(120);
        for (reg, value) in [
            (0xfe, 0),
            (0xc4, 0x80),
            (0x3a, 0x55),
            (0x53, 0x20),
            (0x63, 0xff),
            (0x51, 0),
            (0x58, 0),
            (0x36, 0),
        ] {
            s.reg(reg, &[value]);
        }
        s.reg(0x20, &[]);
        s.reg(0x28, &[]);
        s
    }
    fn reg(&mut self, reg: u8, data: &[u8]) {
        self.spi
            .as_mut()
            .unwrap()
            .half_duplex_write(
                DataMode::SingleTwoDataLines,
                Command::_8Bit(0x02, DataMode::SingleTwoDataLines),
                Address::_24Bit(u32::from(reg) << 8, DataMode::SingleTwoDataLines),
                0,
                data,
            )
            .expect("display register write");
    }
    pub fn show(&mut self) {
        self.reg(0x29, &[]);
        self.reg(0x51, &[128]);
    }
    pub fn flush(&mut self, fb: &[u16], rows: &Rows, mut poll: impl FnMut()) {
        for y in (0..W).step_by(16) {
            let h = 16.min(W - y);
            let Some((x, w)) = rows.band(y, h) else {
                continue;
            };
            assert!(x % 2 == 0 && w % 2 == 0);
            let xa = (x + 6) as u16;
            let xb = (x + w + 5) as u16;
            let ya = y as u16;
            let yb = (y + h - 1) as u16;
            self.reg(0x2a, &[(xa >> 8) as u8, xa as u8, (xb >> 8) as u8, xb as u8]);
            self.reg(0x2b, &[(ya >> 8) as u8, ya as u8, (yb >> 8) as u8, yb as u8]);
            self.reg(0x2c, &[]);
            let mut buf = self.pixels.take().unwrap();
            buf.set_length(16000);
            let target = buf.as_mut_slice();
            let mut n = 0;
            for yy in y..y + h {
                let row = &fb[yy * W + x..yy * W + x + w];
                // SAFETY: ESP32-S3 is little endian; framebuffer words deliberately store
                // big-endian RGB565 on the wire. This read-only byte view is fully in bounds.
                let bytes = unsafe {
                    core::slice::from_raw_parts(row.as_ptr().cast::<u8>(), row.len() * 2)
                };
                target[n..n + bytes.len()].copy_from_slice(bytes);
                n += bytes.len();
            }
            buf.set_length(n);
            let spi = self.spi.take().unwrap();
            let transfer = spi
                .half_duplex_write_buffer(
                    DataMode::Quad,
                    Command::_8Bit(0x32, DataMode::SingleTwoDataLines),
                    Address::_24Bit(0x003c00, DataMode::SingleTwoDataLines),
                    0,
                    n,
                    buf,
                )
                .unwrap_or_else(|(error, _, _)| panic!("display DMA: {error:?}"));
            while !transfer.is_done() {
                poll();
            }
            let (spi, buf) = transfer.wait();
            self.spi = Some(spi);
            self.pixels = Some(buf);
            poll();
        }
    }
}
