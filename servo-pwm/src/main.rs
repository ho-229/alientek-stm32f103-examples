#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_semihosting as _;
use stm32f1xx_hal::{
    gpio::PinState,
    pac,
    prelude::*,
    timer::{Tim2NoRemap, Timer},
};

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

    let mut afio = dp.AFIO.constrain();

    // Acquire the GPIO peripherals
    let mut gpioa = dp.GPIOA.split();
    let mut gpioc = dp.GPIOC.split();

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).delay();

    // Create an array of LEDS to blink
    let mut leds = push_pull_pin_array!(
        [gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5, gpioc.pc6, gpioc.pc7],
        gpioc.crl,
        PinState::High
    );

    let servo_pin = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let mut pwm_channel = dp
        .TIM2
        .pwm_hz::<Tim2NoRemap, _, _>(servo_pin, &mut afio.mapr, 50.Hz(), &clocks)
        .split();

    let max_duty = pwm_channel.get_max_duty() / 8; // 2.5ms
    let min_duty = pwm_channel.get_max_duty() / 40; // 0.5ms
    let step = (max_duty - min_duty) / 8;

    pwm_channel.enable();
    pwm_channel.set_duty(min_duty);

    loop {
        for led in leds.iter_mut() {
            led.set_low();
            timer.delay_ms(500u16);
            pwm_channel.set_duty(pwm_channel.get_duty() + step);
        }
        for led in leds.iter_mut().rev() {
            led.set_high();
            timer.delay_ms(500u16);
            pwm_channel.set_duty(pwm_channel.get_duty() - step);
        }
    }
}
