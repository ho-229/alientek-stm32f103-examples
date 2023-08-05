#![no_std]
#![no_main]

use cortex_m::{asm, peripheral};
use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use panic_semihosting as _;
use stm32f1xx_hal::{
    gpio::PinState,
    pac::{self, interrupt, USART1},
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
    timer::Timer,
};

const PROMPT: &[u8; 10] = b"Received: ";

struct Context {
    rx: Rx<USART1>,
    tx: Tx<USART1>,

    buffer: [u8; 1024],
    pos: usize,
}

static mut CTX: Option<Context> = None;

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

    // Configure the syst timer
    let mut timer = Timer::syst(cp.SYST, &clocks).delay();

    // initialize LED
    let mut led = gpioc
        .pc0
        .into_push_pull_output_with_state(&mut gpioc.crl, PinState::High);

    // USART1 on Pins A9 and A10
    let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let pin_rx = gpioa.pa10;

    // Create an interface struct for USART1 with 9600 Baud
    let (tx, mut rx) = Serial::new(
        dp.USART1,
        (pin_tx, pin_rx),
        &mut afio.mapr,
        Config::default(),
        &clocks,
    )
    .split();

    if rx.is_rx_not_empty() {
        _ = rx.read();
    }
    rx.listen();

    let mut ctx = Context {
        rx,
        tx,
        buffer: [0; 1024],
        pos: PROMPT.len(),
    };
    (&mut ctx.buffer[..PROMPT.len()]).clone_from_slice(PROMPT);

    cortex_m::interrupt::free(|_| unsafe {
        CTX.replace(ctx);
    });

    unsafe {
        peripheral::NVIC::unmask(pac::Interrupt::USART1);
    }

    loop {
        // wait for interrupt
        asm::wfi();

        led.set_low();
        timer.delay_ms(100_u16);
        led.set_high();
    }
}

#[interrupt]
unsafe fn USART1() {
    cortex_m::interrupt::free(|_| {
        let Some(ctx) = CTX.as_mut() else { return; };
        if ctx.rx.is_rx_not_empty() {
            on_ready_read(ctx);
        }
    });
}

#[inline]
fn flash_buffer(ctx: &mut Context) {
    ctx.tx.bwrite_all(&ctx.buffer[..ctx.pos]).unwrap();
    ctx.pos = PROMPT.len();
}

#[inline]
fn on_ready_read(ctx: &mut Context) {
    let Ok(w) = ctx.rx.read() else { return; };
    ctx.buffer[ctx.pos] = w;
    ctx.pos += 1;

    if w == b'\n' || ctx.pos >= ctx.buffer.len() - 1 {
        flash_buffer(ctx);
    }
}
