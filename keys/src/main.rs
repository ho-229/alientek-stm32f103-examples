#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{gpio::PinState, pac, prelude::*, time::MicroSeconds, timer::Timer};

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
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();
    let mut gpiod = dp.GPIOD.split();

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).delay();

    // Create an array of LEDS to blink
    let mut leds =
        push_pull_pin_array!([gpioc.pc0, gpioc.pc1, gpioc.pc2], gpioc.crl, PinState::High);

    // Set beep to high (silence)
    let mut beep = gpiob
        .pb8
        .into_push_pull_output_with_state(&mut gpiob.crh, PinState::High);

    let key_0 = gpioc.pc8.into_pull_up_input(&mut gpioc.crh);
    let key_1 = gpioc.pc9.into_pull_up_input(&mut gpioc.crh);
    let key_2 = gpiod.pd2.into_pull_up_input(&mut gpiod.crl);
    let key_up = gpioa.pa0.into_pull_down_input(&mut gpioa.crl);

    loop {
        // anti shake
        timer.delay(MicroSeconds::from_ticks(50_000));

        if key_up.is_high() {
            leds.iter_mut().for_each(|led| led.toggle());

            beep.set_low();
            timer.delay(MicroSeconds::from_ticks(500_000));
            beep.set_high();
        } else {
            [key_0.is_low(), key_1.is_low(), key_2.is_low()]
                .into_iter()
                .enumerate()
                .filter_map(|(i, on)| on.then_some(i))
                .for_each(|i| leds[i].toggle());
        }

        loop {
            if key_0.is_high() && key_1.is_high() && key_2.is_high() && key_up.is_low() {
                beep.set_high();
                break;
            } else {
                beep.set_low();
            }
        }
    }
}
