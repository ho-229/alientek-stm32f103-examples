#![no_std]
#![no_main]

use cortex_m::{asm, peripheral};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{
    gpio::{Edge, ErasedPin, ExtiPin, Input, Output, Pin, PinState, PullDown, PullUp},
    pac::{self, interrupt},
    prelude::*,
    timer::{Configuration, SysDelay, Tim2NoRemap, Timer},
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

macro_rules! debounce {
    ($timer:expr, $pin:expr) => {
        while $pin.is_low() {}
        $timer.delay_ms(50_u16);
        $pin.clear_interrupt_pending_bit();
    };
}

struct Context {
    timer: SysDelay,
    leds: [ErasedPin<Output>; 3],
    // key_up: Pin<'A', 0, Input<PullDown>>,
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
    let mut dbg = dp.DBGMCU;

    // Acquire the GPIO peripherals
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();
    let mut gpiod = dp.GPIOD.split();

    let mut ctx = Context {
        timer: Timer::syst(cp.SYST, &clocks).delay(),
        leds: push_pull_pin_array!([gpioc.pc0, gpioc.pc1, gpioc.pc2], gpioc.crl, PinState::High),
        // key_up: gpioa.pa0.into_pull_down_input(&mut gpioa.crl),
    };

    let key_up = gpioa.pa0.into_pull_up_input(&mut gpioa.crl);
    let other = gpioa.pa1.into_pull_up_input(&mut gpioa.crl);
    let mut pwm_in = Timer::new(dp.TIM2, &clocks).pwm_input::<Tim2NoRemap, _>(
        (key_up, other),
        &mut afio.mapr,
        &mut dbg,
        Configuration::Frequency(72_000.kHz()),
    );

    // config_interrupt!(ctx.key_up, afio, dp.EXTI, Edge::Rising);

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
    cortex_m::interrupt::free(|_| unsafe { todo!() })
}
