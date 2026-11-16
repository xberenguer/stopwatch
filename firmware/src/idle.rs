use core::sync::atomic::{AtomicBool, Ordering};
use esp_hal::{
    gpio::{Event, Input, Io},
    interrupt::{ElevatedRunLevel, Priority, RunLevel, wait_for_interrupt},
    peripherals::{GPIO, SYSTEM},
    timer::systimer::{SystemTimer, Unit, UnitConfig},
};

static WOKE: AtomicBool = AtomicBool::new(false);

pub fn setup(io: &mut Io<'_>) {
    SYSTEM::regs().cpu_per_conf().modify(|_, w| w.cpu_wait_mode_force_on().clear_bit());
    // Keep Instant's counter running while WAITI gates only the CPU clock.
    unsafe { SystemTimer::configure_unit(Unit::Unit0, UnitConfig::Enabled) };
    io.set_interrupt_handler(wake);
}

// Only the two buttons use GPIO interrupts; main and ISR run on the same core.
#[esp_hal::handler(priority = Priority::Priority1)]
fn wake() {
    let gpio = GPIO::regs();
    for pin in [0, 3] {
        gpio.pin(pin).modify(|_, w| unsafe { w.int_ena().bits(0) });
    }
    gpio.status_w1tc().write(|w| unsafe { w.status_w1tc().bits(1 | 1 << 3) });
    WOKE.store(true, Ordering::Release);
}

pub fn wait(start: &mut Input<'_>, lap: &mut Input<'_>) {
    debug_assert!(RunLevel::current().is_thread());
    start.unlisten();
    lap.unlisten();
    start.clear_interrupt();
    lap.clear_interrupt();
    WOKE.store(false, Ordering::Release);
    start.listen(Event::RisingEdge);
    lap.listen(Event::FallingEdge);
    loop {
        // Mask GPIO until WAITI atomically unmasks it: no check-to-sleep race,
        // and no critical-section lock may be held while WAITI enables interrupts.
        let old = unsafe { RunLevel::change(RunLevel::Interrupt(ElevatedRunLevel::Level1)) };
        if WOKE.load(Ordering::Acquire) || start.is_high() || lap.is_low() {
            unsafe { RunLevel::change(old) };
            break;
        }
        wait_for_interrupt();
    }
    start.unlisten();
    lap.unlisten();
    start.clear_interrupt();
    lap.clear_interrupt();
}
