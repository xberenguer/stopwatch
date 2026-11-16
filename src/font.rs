// Packed font: glyph count, 7-byte metrics per glyph, then 4-bit pixel masks.
pub struct Font(&'static [u8]);
pub struct Glyph {
    pub w: usize,
    pub h: usize,
    pub advance: usize,
    pub top: i32,
    pub data: &'static [u8],
}
impl Font {
    pub fn glyph(&self, ch: u8) -> Option<Glyph> {
        let end = 1 + self.0[0] as usize * 7;
        let m = self.0[1..end].chunks_exact(7).find(|m| m[0] == ch)?;
        let offset = end + u16::from_le_bytes([m[5], m[6]]) as usize;
        Some(Glyph {
            w: m[1] as usize,
            h: m[2] as usize,
            advance: m[3] as usize,
            top: m[4] as i8 as i32,
            data: &self.0[offset..],
        })
    }
    pub fn width(&self, text: &str) -> i32 {
        text.bytes().filter_map(|ch| self.glyph(ch)).map(|g| g.advance as i32).sum()
    }
}
pub static LARGE: Font = Font(include_bytes!("../assets/large.bin"));
pub static COMPACT: Font = Font(include_bytes!("../assets/compact.bin"));
pub static SMALL: Font = Font(include_bytes!("../assets/small.bin"));
pub static LABEL: Font = Font(include_bytes!("../assets/label.bin"));
