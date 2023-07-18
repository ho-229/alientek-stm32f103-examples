#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use nb::block;
use panic_semihosting as _;
use stm32f1xx_hal::{
    gpio::PinState,
    pac,
    prelude::*,
    serial::{Config, Error, Serial},
    time::MicroSeconds,
    timer::Timer,
};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIO peripherals
    let mut gpioa = dp.GPIOA.split();
    let mut gpioc = dp.GPIOC.split();

    let mut afio = dp.AFIO.constrain();

    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, &clocks).delay();

    // initialize LED
    let mut led = gpioc
        .pc0
        .into_push_pull_output_with_state(&mut gpioc.crl, PinState::High);

    // USART1 on Pins A9 and A10
    let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let pin_rx = gpioa.pa10;

    // Create an interface struct for USART1 with 9600 Baud
    let (mut tx, mut rx) = Serial::new(
        dp.USART1,
        (pin_tx, pin_rx),
        &mut afio.mapr,
        Config::default(),
        &clocks,
    )
    .split();

    const PROMPT: &[u8; 10] = b"Received: ";

    let mut buf = [0u8; 1024];
    (&mut buf[..10]).copy_from_slice(PROMPT);

    loop {
        let mut read_byte;
        let mut pos = PROMPT.len();

        // reset bits of overrun
        matches!(block!(rx.read()), Ok(_) | Err(Error::Overrun))
            .then_some(())
            .expect("not expected overrun error and ok");

        loop {
            read_byte = block!(rx.read()).unwrap();
            buf[pos] = read_byte;
            pos += 1;

            if read_byte == b'\n' || pos >= buf.len() {
                break;
            }
        }

        tx.bwrite_all(&buf[..pos]).unwrap();

        led.set_low();
        timer.delay(MicroSeconds::from_ticks(100_000));
        led.set_high();
    }
}
