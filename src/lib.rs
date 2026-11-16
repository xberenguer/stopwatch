#![no_std]
use core::fmt::{self, Write};

pub mod render;

/// Largest displayable time: 99 hours, 59 minutes, 59.99 seconds.
pub const MAX_US: u64 = 359_999_990_000;
const DEBOUNCE_US: u64 = 12_000;
const HOLD_US: u64 = 600_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    StartPause,
    LapReset,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Stopwatch {
    accumulated_us: u64,
    started_us: Option<u64>,
    pub lap_us: Option<u64>,
    pub lap_count: u32,
    last_lap_elapsed_us: u64,
}

impl Stopwatch {
    pub fn running(&self) -> bool {
        self.started_us.is_some()
    }

    pub fn elapsed_us(&self, now_us: u64) -> u64 {
        self.accumulated_us
            .saturating_add(self.started_us.map_or(0, |start| now_us.saturating_sub(start)))
            .min(MAX_US)
    }

    pub fn press(&mut self, button: Button, now_us: u64) {
        self.tick(now_us);
        match button {
            Button::StartPause => {
                if self.running() {
                    self.accumulated_us = self.elapsed_us(now_us);
                    self.started_us = None;
                } else if self.accumulated_us < MAX_US {
                    self.started_us = Some(now_us);
                }
            }
            Button::LapReset if self.running() => {
                let elapsed = self.elapsed_us(now_us);
                self.lap_us = Some(elapsed - self.last_lap_elapsed_us);
                self.last_lap_elapsed_us = elapsed;
                self.lap_count = self.lap_count.saturating_add(1);
            }
            Button::LapReset => *self = Self::default(),
        }
    }

    pub fn tick(&mut self, now_us: u64) {
        if self.running() && self.elapsed_us(now_us) == MAX_US {
            self.accumulated_us = MAX_US;
            self.started_us = None;
        }
    }
}

/// Hold-to-off is available only when the press begins stopped at zero.
pub struct Controls {
    pub watch: Stopwatch,
    pub screen_on: bool,
    hold_us: Option<u64>,
}
impl Default for Controls {
    fn default() -> Self {
        Self { watch: Stopwatch::default(), screen_on: true, hold_us: None }
    }
}
impl Controls {
    pub fn press(&mut self, button: Button, now_us: u64) {
        if !self.screen_on {
            self.screen_on = true;
            if button == Button::StartPause {
                self.watch.press(button, now_us);
            }
        } else if button == Button::StartPause
            || self.watch.running()
            || self.watch.elapsed_us(now_us) != 0
        {
            self.hold_us = None;
            self.watch.press(button, now_us);
        } else {
            self.hold_us = Some(now_us);
        }
    }
    pub fn tick(&mut self, now_us: u64, small_held: bool) {
        if !small_held {
            self.hold_us = None;
        } else if self.hold_us.is_some_and(|t| now_us.saturating_sub(t) >= HOLD_US) {
            self.hold_us = None;
            self.screen_on = false;
        }
        self.watch.tick(now_us);
    }
}

/// Accept an edge after 12 ms of stability, preserving its first observed time.
/// A held key at startup must be released before it can generate a press.
#[derive(Default)]
pub struct Debouncer {
    raw: bool,
    stable: Option<bool>,
    changed_us: u64,
}

impl Debouncer {
    pub fn poll(&mut self, pressed: bool, now_us: u64) -> Option<u64> {
        if self.stable.is_none() {
            self.raw = pressed;
            self.stable = Some(pressed);
            self.changed_us = now_us;
            return None;
        }
        if pressed != self.raw {
            self.raw = pressed;
            self.changed_us = now_us;
        }
        if self.stable != Some(self.raw) && now_us.saturating_sub(self.changed_us) >= DEBOUNCE_US {
            self.stable = Some(self.raw);
            return self.raw.then_some(self.changed_us);
        }
        None
    }
}

/// Fixed-capacity, allocation-free ASCII time; hundredths are truncated.
pub fn format_time(us: u64) -> Text {
    let cs = us.min(MAX_US) / 10_000;
    let seconds = cs / 100;
    let hours = seconds / 3600;
    let mut t = Text::default();
    if hours > 0 {
        write!(t, "{hours:02}:").unwrap();
    }
    write!(t, "{:02}:{:02}.{:02}", seconds / 60 % 60, seconds % 60, cs % 100).unwrap();
    t
}

/// Enough stack space for the longest time or "LAP 4294967295"; no allocator.
#[derive(Default)]
pub struct Text {
    bytes: [u8; 14],
    len: usize,
}
impl Text {
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap()
    }
}
impl Write for Text {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?.copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/unit/lib.rs"]
mod tests;
