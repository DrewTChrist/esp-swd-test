#![no_std]
#![no_main]

mod swd;
use swd::Swd;

use esp32_hal::{
    clock::ClockControl, gpio::IO, peripherals::Peripherals, prelude::*, timer::TimerGroup, Delay,
    Rtc,
};
use esp_backtrace as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt = timer_group0.wdt;
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    wdt.disable();
    rtc.rwdt.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let clock_pin = io.pins.gpio21.into_push_pull_output();
    let data_pin = io.pins.gpio22.into_push_pull_output();
    let delay = Delay::new(&clocks);
    let mut swd = Swd::new(clock_pin, data_pin, delay);

    let mut led = io.pins.gpio13.into_push_pull_output();
    led.set_high().unwrap();

    loop {
        led.toggle().unwrap();
        for _ in 0..2000 {}
        //delay.delay_ms(1000u32);
    }
}
