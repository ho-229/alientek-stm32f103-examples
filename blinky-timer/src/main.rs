#![no_std]
#![no_main]

use cortex_m::{asm, peripheral};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{
    gpio::{ErasedPin, Output, PinState},
    pac::{self, interrupt},
    prelude::*,
    timer::{CounterMs, Event},
};

macro_rules! push_pull_pin_array {
    ([$($pin:expr),+], $cr:expr, $state:expr) => {
        [$($pin.into_push_pull_output_with_state(&mut $cr, $state).erase(),)+]
    };
}

struct Context {
    index: usize,
    leds: [ErasedPin<Output>; 8],
    timer: CounterMs<pac::TIM2>,
}

static mut CTX: Option<Context> = None;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIO peripherals
    let mut gpioc = dp.GPIOC.split();

    // Create an array of LEDS to blink
    let leds = push_pull_pin_array!(
        [gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5, gpioc.pc6, gpioc.pc7],
        gpioc.crl,
        PinState::High
    );

    let mut timer = dp.TIM2.counter_ms(&clocks);
    let _ = timer.start(500.millis());
    timer.listen(Event::Update);

    let ctx = Context {
        index: 0,
        leds,
        timer,
    };

    cortex_m::interrupt::free(|_| unsafe {
        CTX.replace(ctx);
    });

    unsafe {
        peripheral::NVIC::unmask(pac::Interrupt::TIM2);
    }

    loop {
        // wait for interrupt
        asm::wfi();
    }
}

#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|_| unsafe {
        let Some(ctx) = CTX.as_mut() else { return; };
        ctx.leds[ctx.index].toggle();
        ctx.index = (ctx.index + 1) % ctx.leds.len();

        ctx.timer.clear_interrupt(Event::Update);
    })
}
