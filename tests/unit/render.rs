extern crate std;
use super::*;
#[test]
fn rotation_matches_seeder_canvas_and_is_bijective() {
    assert_eq!(native_xy(0, 0), (465, 0));
    assert_eq!(native_xy(465, 0), (465, 465));
    for y in 0..W {
        for x in 0..W {
            let (nx, ny) = native_xy(x, y);
            assert_eq!((ny, W - 1 - nx), (x, y));
        }
    }
}
#[test]
fn every_time_format_fits_the_centered_number_area() {
    for value in ["00:00.00", "59:59.99"] {
        assert!(font::LARGE.width(value) <= 410);
    }
    assert!(font::COMPACT.width("99:59:59.99") <= 410);
}

#[test]
fn panel_windows_are_even_aligned_and_cover_all_changed_columns() {
    // Exercise every possible one-pixel span, including both screen edges.
    // Odd-start / odd-width windows corrupt CO5300 RGB565 writes on hardware.
    for x in 0..W {
        let mut rows = Rows::default();
        rows.mark(x, 7);
        let (start, width) = rows.band(0, 16).unwrap();
        assert_eq!(start % 2, 0);
        assert_eq!(width % 2, 0);
        assert!(start <= x && start + width > x && start + width <= W);
    }
    let mut rows = Rows::default();
    assert_eq!(rows.band(0, 16), None);
    rows.full();
    assert_eq!(rows.band(0, 16), Some((0, W)));
}

#[test]
fn incremental_frames_match_fresh_frames_after_controls_and_hour_transition() {
    use crate::Button;
    let mut watch = Stopwatch::default();
    let mut renderer = Renderer::default();
    let mut fb = std::vec![0; PIXELS];
    let mut rows = Rows::default();
    let steps = [
        (None, 0),
        (Some(Button::StartPause), 10_000),
        (None, 83_470_000),
        (Some(Button::LapReset), 83_480_000),
        (Some(Button::LapReset), 95_000_000),
        (Some(Button::StartPause), 96_000_000),
        (Some(Button::StartPause), 200_000_000),
        (None, 4_000_000_000),
        (Some(Button::StartPause), 4_010_000_000),
        (Some(Button::LapReset), 4_020_000_000),
    ];
    for (button, now) in steps {
        if let Some(button) = button {
            watch.press(button, now);
        }
        rows.clear();
        renderer.draw(&mut fb, &mut rows, &watch, now, || {});
        let mut fresh = std::vec![0; PIXELS];
        Renderer::default().draw(&mut fresh, &mut Rows::default(), &watch, now, || {});
        assert!(fb == fresh, "stale pixels at {now}");
    }
}
