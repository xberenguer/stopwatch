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
    write!(file, "P6\n{W} {W}\n255\n").unwrap();
    for y in 0..W {
        for x in 0..W {
            let (nx, ny) = native_xy(x, y);
            let c = u16::from_be(fb[ny * W + nx]);
            let rgb = [
                ((c >> 11) * 255 / 31) as u8,
                (((c >> 5) & 63) * 255 / 63) as u8,
                ((c & 31) * 255 / 31) as u8,
            ];
            file.write_all(&rgb).unwrap();
        }
    }
}
fn main() {
    let out = env::args().nth(1).unwrap_or("previews".into());
    let out = Path::new(&out);
    std::fs::create_dir_all(out).unwrap();
    let mut s = Stopwatch::default();
    save(out, "ready", &s, 0);
    s.press(Button::StartPause, 0);
    save(out, "running", &s, 83_470_000);
    s.press(Button::LapReset, 74_320_000);
    save(out, "lap", &s, 83_470_000);
    s.press(Button::StartPause, 83_470_000);
    save(out, "paused", &s, 100_000_000);
    let mut long = Stopwatch::default();
    long.press(Button::StartPause, 0);
    save(out, "hours", &long, 45_296_780_000);
}
