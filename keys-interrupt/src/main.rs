#![no_std]
#![no_main]

use cortex_m::{asm, peripheral};
use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use panic_halt as _;
use stm32f1xx_hal::{
    gpio::{Edge, ErasedPin, ExtiPin, Input, Output, Pin, PinState, PullDown, PullUp},
    pac::{self, interrupt},
    prelude::*,
    timer::{SysDelay, Timer},
};

macro_rules! push_pull_pin_array {
    ([$($pin:expr),+], $cr:expr, $state:expr) => {
        [$($pin.into_push_pull_output_with_state(&mut $cr, $state).erase(),)+]
    };
}

macro_rules! config_interrupt {
    ($pin:expr, $afio:expr, $channel:expr, $edge:expr) => {
        $pin.make_interrupt_source(&mut $afio);
        $pin.trigger_on_edge(&mut $channel, $edge);
        $pin.enable_interrupt(&mut $channel);
    };
}

struct Context {
    timer: SysDelay,
    leds: [ErasedPin<Output>; 3],
    beep: Pin<'B', 8, Output>,
    key_0: Pin<'C', 8, Input<PullUp>>,
    key_1: Pin<'C', 9, Input<PullUp>>,
    key_2: Pin<'D', 2, Input<PullUp>>,
    key_up: Pin<'A', 0, Input<PullDown>>,
}

static mut CTX: Option<Context> = None;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain();

    // Acquire the GPIO peripherals
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();
    let mut gpiod = dp.GPIOD.split();

    let mut ctx = Context {
        timer: Timer::syst(cp.SYST, &clocks).delay(),
        leds: push_pull_pin_array!([gpioc.pc0, gpioc.pc1, gpioc.pc2], gpioc.crl, PinState::High),
        beep: gpiob
            .pb8
            .into_push_pull_output_with_state(&mut gpiob.crh, PinState::High),
        key_0: gpioc.pc8.into_pull_up_input(&mut gpioc.crh),
        key_1: gpioc.pc9.into_pull_up_input(&mut gpioc.crh),
        key_2: gpiod.pd2.into_pull_up_input(&mut gpiod.crl),
        key_up: gpioa.pa0.into_pull_down_input(&mut gpioa.crl),
    };

    config_interrupt!(ctx.key_0, afio, dp.EXTI, Edge::Falling);
    config_interrupt!(ctx.key_1, afio, dp.EXTI, Edge::Falling);
    config_interrupt!(ctx.key_2, afio, dp.EXTI, Edge::Falling);
    config_interrupt!(ctx.key_up, afio, dp.EXTI, Edge::Rising);

    cortex_m::interrupt::free(|_| unsafe {
        CTX.replace(ctx);
    });

    unsafe {
        peripheral::NVIC::unmask(pac::Interrupt::EXTI0);
        peripheral::NVIC::unmask(pac::Interrupt::EXTI2);
        peripheral::NVIC::unmask(pac::Interrupt::EXTI9_5);
    }

    loop {
        // wait for interrupt
        asm::wfi();
    }
}

/// key up
#[interrupt]
unsafe fn EXTI0() {
    cortex_m::interrupt::free(|_| {
        let Some(ctx) = CTX.as_mut() else { return; };
        ctx.leds.iter_mut().for_each(|led| led.toggle());

        ctx.beep.set_low();
        ctx.timer.delay_ms(500_u16);
        ctx.beep.set_high();

        ctx.key_up.clear_interrupt_pending_bit();
    });
}

/// key 2
#[interrupt]
unsafe fn EXTI2() {
    cortex_m::interrupt::free(|_| {
        let Some(ctx) = CTX.as_mut() else { return; };
        ctx.leds[2].toggle();
        ctx.timer.delay_ms(10_u16);
        ctx.key_2.clear_interrupt_pending_bit();
    })
}

/// key 0 and key 1
#[interrupt]
unsafe fn EXTI9_5() {
    cortex_m::interrupt::free(|_| {
        let Some(ctx) = CTX.as_mut() else { return; };
        if ctx.key_0.check_interrupt() {
            ctx.leds[0].toggle();
            ctx.key_0.clear_interrupt_pending_bit();
        } else if ctx.key_1.check_interrupt() {
            ctx.leds[1].toggle();
            ctx.key_1.clear_interrupt_pending_bit();
        }
        ctx.timer.delay_ms(10_u16);
    })
}
