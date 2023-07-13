#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use nb::block;
use panic_halt as _;
use stm32f1xx_hal::{gpio::PinState, pac, prelude::*, timer::Timer};

macro_rules! push_pull_pin_array {
    ([$($pin:expr),+], $cr:expr, $state:expr) => {
        [$($pin.into_push_pull_output_with_state(&mut $cr, $state).erase(),)+]
    };
}

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIO peripherals
    let mut gpioc = dp.GPIOC.split();

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(3.Hz()).unwrap();

    // Create an array of LEDS to blink
    let mut leds = push_pull_pin_array!(
        [gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5, gpioc.pc6, gpioc.pc7],
        gpioc.crl,
        PinState::High
    );

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        for led in leds.iter_mut() {
            led.set_low();
            block!(timer.wait()).unwrap();
        }
        for led in leds.iter_mut() {
            led.set_high();
            block!(timer.wait()).unwrap();
        }
    }
}
