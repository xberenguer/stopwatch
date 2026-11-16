use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use stopwatch::{
    Button, Stopwatch,
    render::{PIXELS, Renderer, Rows, W, native_xy},
};

fn save(out: &Path, name: &str, watch: &Stopwatch, now: u64) {
    let mut fb = vec![0; PIXELS];
    Renderer::default().draw(&mut fb, &mut Rows::default(), watch, now, || {});
    let mut file = BufWriter::new(File::create(out.join(format!("{name}.ppm"))).unwrap());
    let size = W + 64;
    let background = [19, 24, 30];
    write!(file, "P6\n{size} {size}\n255\n").unwrap();
    for y in 0..size {
        for x in 0..size {
            let mut rgb = background;
            if (32..W + 32).contains(&x) && (32..W + 32).contains(&y) {
                let (sx, sy) = (x - 32, y - 32);
                let (nx, ny) = native_xy(sx, sy);
                let c = u16::from_be(fb[ny * W + nx]);
                let screen = [
                    ((c >> 11) * 255 / 31) as u8,
                    (((c >> 5) & 63) * 255 / 63) as u8,
                    ((c & 31) * 255 / 31) as u8,
                ];
                let dx = sx as f32 + 0.5 - W as f32 / 2.0;
                let dy = sy as f32 + 0.5 - W as f32 / 2.0;
                let coverage = (W as f32 / 2.0 - dx.hypot(dy) + 0.5).clamp(0.0, 1.0);
                for i in 0..3 {
                    rgb[i] = (f32::from(screen[i]) * coverage
                        + f32::from(background[i]) * (1.0 - coverage))
                        .round() as u8;
                }
            }
            file.write_all(&rgb).unwrap();
        }
    }
}
fn main() {
    let out = env::args().nth(1).unwrap_or("previews".into());
    let out = Path::new(&out);
    std::fs::create_dir_all(out).unwrap();
    let mut s = Stopwatch::default();
    save(out, "ready-charcoal", &s, 0);
    s.press(Button::StartPause, 0);
    save(out, "running-charcoal", &s, 83_470_000);
    s.press(Button::LapReset, 74_320_000);
    save(out, "lap-charcoal", &s, 83_470_000);
    s.press(Button::StartPause, 83_470_000);
    save(out, "paused-charcoal", &s, 100_000_000);
    let mut long = Stopwatch::default();
    long.press(Button::StartPause, 0);
    save(out, "hours-charcoal", &long, 45_296_780_000);
    if env::args().any(|arg| arg == "--animate") {
        let frames = out.join("frames");
        std::fs::create_dir_all(&frames).unwrap();
        // A real running total and a fixed lap interval, rendered by the firmware code.
        let mut running = Stopwatch::default();
        running.press(Button::StartPause, 0);
        running.press(Button::LapReset, 74_320_000);
        for frame in 0..80 {
            save(&frames, &format!("{frame:03}"), &running, 83_000_000 + frame * 50_000);
        }
    }
}
