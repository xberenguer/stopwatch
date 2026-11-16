use super::*;

#[test]
fn pauses_and_resumes_without_counting_paused_time() {
    let mut s = Stopwatch::default();
    s.press(Button::StartPause, 1_000_000);
    s.press(Button::StartPause, 2_234_567);
    assert_eq!(s.elapsed_us(9_000_000), 1_234_567);
    s.press(Button::StartPause, 10_000_000);
    assert_eq!(s.elapsed_us(11_000_000), 2_234_567);
}

#[test]
fn laps_measure_intervals_and_do_not_interrupt_total() {
    let mut s = Stopwatch::default();
    s.press(Button::StartPause, 0);
    s.press(Button::LapReset, 2_000_000);
    assert_eq!(s.lap_us, Some(2_000_000));
    s.press(Button::LapReset, 5_000_000);
    assert_eq!(s.lap_us, Some(3_000_000));
    assert_eq!(s.lap_count, 2);
    assert_eq!(s.elapsed_us(6_000_000), 6_000_000);
}

#[test]
fn lap_excludes_pause_and_reset_clears_everything() {
    let mut s = Stopwatch::default();
    s.press(Button::StartPause, 0);
    s.press(Button::LapReset, 1_000_000);
    s.press(Button::StartPause, 2_000_000);
    s.press(Button::StartPause, 9_000_000);
    s.press(Button::LapReset, 10_000_000);
    assert_eq!(s.lap_us, Some(2_000_000));
    s.press(Button::StartPause, 11_000_000);
    s.press(Button::LapReset, 12_000_000);
    assert_eq!(s.elapsed_us(20_000_000), 0);
    assert!(!s.running());
    assert_eq!(s.lap_us, None);
    assert_eq!(s.lap_count, 0);
}

#[test]
fn reaches_limit_without_wrapping_or_restarting() {
    let mut s = Stopwatch::default();
    s.press(Button::StartPause, 0);
    s.tick(MAX_US + 1);
    assert_eq!(s.elapsed_us(u64::MAX), MAX_US);
    assert!(!s.running());
    s.press(Button::StartPause, MAX_US + 2);
    assert!(!s.running());
    s.press(Button::LapReset, MAX_US + 3);
    s.press(Button::StartPause, MAX_US + 4);
    assert!(s.running());
}

#[test]
fn formatting_crosses_minute_and_hour_boundaries() {
    for (us, expected) in [
        (0, "00:00.00"),
        (9_999, "00:00.00"),
        (59_999_999, "00:59.99"),
        (60_000_000, "01:00.00"),
        (3_600_000_000, "01:00:00.00"),
        (MAX_US, "99:59:59.99"),
    ] {
        assert_eq!(format_time(us).as_str(), expected);
    }
}

#[test]
fn bouncing_and_holding_produce_one_timestamped_press() {
    let mut d = Debouncer::default();
    assert_eq!(d.poll(false, 0), None);
    assert_eq!(d.poll(true, 1_000), None);
    assert_eq!(d.poll(false, 3_000), None);
    assert_eq!(d.poll(true, 5_000), None);
    assert_eq!(d.poll(true, 16_999), None);
    assert_eq!(d.poll(true, 17_000), Some(5_000));
    assert_eq!(d.poll(true, 9_000_000), None);
    assert_eq!(d.poll(false, 9_001_000), None);
    assert_eq!(d.poll(false, 9_013_000), None);
    assert_eq!(d.poll(true, 9_020_000), None);
    assert_eq!(d.poll(true, 9_032_000), Some(9_020_000));
}

#[test]
fn boot_held_key_is_ignored_until_released() {
    let mut d = Debouncer::default();
    assert_eq!(d.poll(true, 0), None);
    assert_eq!(d.poll(true, 1_000_000), None);
    d.poll(false, 2_000_000);
    d.poll(false, 2_012_000);
    d.poll(true, 3_000_000);
    assert_eq!(d.poll(true, 3_012_000), Some(3_000_000));
}
