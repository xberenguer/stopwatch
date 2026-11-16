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

#[test]
fn holding_small_at_zero_hides_at_six_hundred_milliseconds() {
    let mut c = Controls::default();
    c.press(Button::LapReset, 2_000_000);
    c.tick(2_599_999, true);
    assert!(c.screen_on);
    c.tick(2_600_000, true);
    assert!(!c.screen_on);
    assert!(!c.watch.running());
    assert_eq!(c.watch.elapsed_us(3_000_000), 0);
}

#[test]
fn either_button_wakes_but_only_start_begins_timing() {
    let mut c = Controls::default();
    c.press(Button::LapReset, 2_000_000);
    c.tick(2_600_000, true);
    assert!(!c.screen_on);
    c.press(Button::LapReset, 10_000_000);
    c.tick(10_600_000, true);
    assert!(c.screen_on);
    assert!(!c.watch.running());
    assert_eq!(c.watch.elapsed_us(10_000_000), 0);
    c.tick(10_700_000, false);
    c.press(Button::LapReset, 11_000_000);
    c.tick(11_600_000, true);
    assert!(!c.screen_on);
    c.press(Button::StartPause, 3_600_000_000);
    c.tick(3_600_000_000, false);
    assert!(c.screen_on);
    assert!(c.watch.running());
    assert_eq!(c.watch.elapsed_us(3_600_000_000), 0);
    assert_eq!(c.watch.elapsed_us(3_601_234_567), 1_234_567);
}

#[test]
fn running_small_presses_record_laps_immediately_and_holding_does_not_hide() {
    let mut c = Controls::default();
    c.press(Button::StartPause, 1_000_000);
    c.press(Button::LapReset, 2_000_000);
    assert_eq!(c.watch.lap_count, 1);
    assert_eq!(c.watch.lap_us, Some(1_000_000));
    c.press(Button::LapReset, 2_200_000);
    assert_eq!(c.watch.lap_count, 2);
    assert_eq!(c.watch.lap_us, Some(200_000));
    c.tick(3_000_000, true);
    assert!(c.screen_on);
    assert!(c.watch.running());
    assert_eq!(c.watch.elapsed_us(2_312_001), 1_312_001);
}

#[test]
fn holding_a_nonzero_reset_does_not_hide_until_a_fresh_zero_press() {
    let mut c = Controls::default();
    c.press(Button::StartPause, 0);
    c.press(Button::LapReset, 500_000);
    c.press(Button::StartPause, 1_000_000);
    c.press(Button::LapReset, 2_000_000);
    assert_eq!(c.watch.elapsed_us(2_000_000), 0);
    assert_eq!(c.watch.lap_count, 0);
    assert_eq!(c.watch.lap_us, None);
    assert!(c.screen_on);
    assert!(!c.watch.running());
    c.tick(2_700_000, true);
    assert!(c.screen_on);
    c.tick(2_800_000, false);
    c.press(Button::LapReset, 3_000_000);
    c.tick(3_600_000, true);
    assert!(!c.screen_on);
}

#[test]
fn short_zero_taps_do_not_hide_or_accumulate_hold_time() {
    let mut c = Controls::default();
    c.press(Button::LapReset, 2_000_000);
    c.tick(2_400_000, false);
    c.press(Button::LapReset, 2_500_000);
    c.tick(2_900_000, true);
    c.tick(3_000_000, false);
    c.tick(4_000_000, false);
    assert!(c.screen_on);
    assert!(!c.watch.running());
}

#[test]
fn releasing_cancels_a_hold_until_another_press() {
    let mut c = Controls::default();
    c.press(Button::LapReset, 1_000_000);
    c.tick(1_599_999, false);
    c.tick(1_600_000, true);
    assert!(c.screen_on);
    assert_eq!(c.watch.elapsed_us(1_600_000), 0);
    assert!(!c.watch.running());
}

#[test]
fn start_cancels_a_zero_screen_hold() {
    let mut c = Controls::default();
    c.press(Button::LapReset, 1_000_000);
    c.press(Button::StartPause, 1_100_000);
    c.tick(1_700_000, true);
    assert!(c.screen_on);
    assert!(c.watch.running());
    assert_eq!(c.watch.elapsed_us(1_700_000), 600_000);
}

#[test]
fn immediate_running_lap_at_limit_does_not_turn_into_a_reset() {
    let mut c = Controls::default();
    c.press(Button::StartPause, 0);
    c.press(Button::LapReset, MAX_US - 100_000);
    c.tick(MAX_US, true);
    c.tick(MAX_US + 600_000, true);
    assert_eq!(c.watch.elapsed_us(MAX_US + 300_000), MAX_US);
    assert!(!c.watch.running());
    assert_eq!(c.watch.lap_count, 1);
    assert_eq!(c.watch.lap_us, Some(MAX_US - 100_000));
}
