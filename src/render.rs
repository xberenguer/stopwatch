use crate::{Stopwatch, Text, format_time};
use core::fmt::Write;
#[path = "font.rs"]
mod font;
use font::Font;

pub const W: usize = 466;
pub const PIXELS: usize = W * W;

/// Changed columns on each native panel row.
pub struct Rows {
    lo: [u16; W],
    hi: [u16; W],
}
impl Default for Rows {
    fn default() -> Self {
        Self { lo: [W as u16; W], hi: [0; W] }
    }
}
impl Rows {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn full(&mut self) {
        self.lo.fill(0);
        self.hi.fill(W as u16);
    }
    fn mark(&mut self, x: usize, y: usize) {
        self.lo[y] = self.lo[y].min(x as u16);
        self.hi[y] = self.hi[y].max((x + 1) as u16);
    }
    pub fn band(&self, y: usize, h: usize) -> Option<(usize, usize)> {
        let x = *self.lo[y..y + h].iter().min()? as usize;
        let end = *self.hi[y..y + h].iter().max()? as usize;
        if end <= x {
            return None;
        }
        // CO5300 requires even starts and widths. Expand to cover changed pixels.
        let start = x & !1;
        Some((start, ((end + 1) & !1).min(W) - start))
    }
}

/// Seeder canvas rotation 1: pixels CW90, enclosure CW270, buttons on top.
pub const fn native_xy(x: usize, y: usize) -> (usize, usize) {
    (W - 1 - y, x)
}

struct Canvas<'a, F> {
    fb: &'a mut [u16],
    rows: &'a mut Rows,
    poll: F,
}
impl<F: FnMut()> Canvas<'_, F> {
    fn pixel(&mut self, x: i32, y: i32, gray: u8) {
        if !(0..W as i32).contains(&x) || !(0..W as i32).contains(&y) {
            return;
        }
        let (x, y) = native_xy(x as usize, y as usize);
        let g = u16::from(gray);
        let color = (((g >> 3) << 11) | ((g >> 2) << 5) | (g >> 3)).to_be();
        if self.fb[y * W + x] != color {
            self.fb[y * W + x] = color;
            self.rows.mark(x, y);
        }
    }
    fn clear(&mut self, x: i32, y: i32, w: i32, h: i32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.pixel(xx, yy, 0);
            }
            (self.poll)();
        }
    }
    fn text(&mut self, font: &Font, value: &str, at: (i32, i32), gray: u8, dim: bool) {
        let mut x = at.0 - font.width(value) / 2;
        let mut fraction = false;
        for ch in value.bytes() {
            if ch == b'.' {
                fraction = dim;
            }
            let Some(g) = font.glyph(ch) else {
                continue;
            };
            let dx = x + (g.advance as i32 - g.w as i32) / 2;
            for y in 0..g.h {
                for xx in 0..g.w {
                    let i = y * g.w + xx;
                    let packed = g.data[i / 2];
                    let alpha = if i % 2 == 0 { packed >> 4 } else { packed & 15 };
                    if alpha > 0 {
                        let shade = if fraction { 148 } else { gray };
                        self.pixel(
                            dx + xx as i32,
                            at.1 + g.top + y as i32,
                            (u16::from(shade) * u16::from(alpha) / 15) as u8,
                        );
                    }
                }
                (self.poll)();
            }
            x += g.advance as i32;
        }
    }
}

#[derive(Default)]
pub struct Renderer {
    last_cs: Option<u64>,
    last_buttons: Option<(bool, bool)>,
    last_laps: u32,
}
impl Renderer {
    pub fn draw(
        &mut self,
        fb: &mut [u16],
        rows: &mut Rows,
        watch: &Stopwatch,
        now_us: u64,
        poll: impl FnMut(),
    ) {
        assert_eq!(fb.len(), PIXELS);
        let mut c = Canvas { fb, rows, poll };
        let elapsed = watch.elapsed_us(now_us);
        if self.last_cs != Some(elapsed / 10_000) {
            c.clear(28, 185, 410, 96);
            let (font, baseline) =
                if elapsed >= 3_600_000_000 { (&font::COMPACT, 253) } else { (&font::LARGE, 262) };
            c.text(font, format_time(elapsed).as_str(), (233, baseline), 242, true);
            self.last_cs = Some(elapsed / 10_000);
        }
        let buttons = (watch.running(), elapsed == 0);
        if self.last_buttons != Some(buttons) {
            c.clear(92, 98, 282, 28);
            let action = if watch.running() {
                "PAUSE"
            } else if elapsed > 0 {
                "RESUME"
            } else {
                "START"
            };
            c.text(&font::LABEL, action, (155, 118), 108, false);
            c.text(
                &font::LABEL,
                if watch.running() { "LAP" } else { "RESET" },
                (311, 118),
                if elapsed == 0 { 58 } else { 108 },
                false,
            );
            self.last_buttons = Some(buttons);
        }
        if self.last_laps != watch.lap_count {
            c.clear(75, 306, 316, 70);
            if let Some(lap) = watch.lap_us {
                let mut label = Text::default();
                write!(label, "LAP {}", watch.lap_count).unwrap();
                c.text(&font::LABEL, label.as_str(), (233, 325), 88, false);
                c.text(&font::SMALL, format_time(lap).as_str(), (233, 356), 158, false);
            }
            self.last_laps = watch.lap_count;
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/render.rs"]
mod tests;
