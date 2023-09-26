#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};
use core::{borrow::BorrowMut, cell::RefCell};

use embedded_hal::{blocking::delay::DelayMs, digital::v2::OutputPin};

use critical_section::Mutex;

use rp2040_hal::entry;

use fugit::RateExtU32;

use ssd1306::mode::DisplayConfig;
use ssd1306::rotation::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::I2CDisplayInterface;
use ssd1306::Ssd1306;

use inverted_pin::InvertedPin;

use board::{hal, Pins, XOSC_CRYSTAL_FREQ};
use hal::{gpio, gpio::Pin, pac, Timer, I2C};
use seeeduino_xiao_rp2040 as board;

struct Shared<T>(Mutex<RefCell<T>>);

impl<T> Shared<T> {
    const fn new(t: T) -> Self {
        Self(Mutex::new(RefCell::new(t)))
    }

    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        critical_section::with(|cs| f(self.0.borrow(cs).borrow_mut().borrow_mut()))
    }
}

impl<T> Shared<Option<T>> {
    const fn empty() -> Self {
        Self::new(None)
    }

    fn replace(&self, t: T) -> Option<T> {
        self.with(|o| o.replace(t))
    }

    fn take(&self) -> Option<T> {
        self.with(|o| o.take())
    }
}

static GLOBAL_BLUE_LED: Shared<
    Option<InvertedPin<Pin<gpio::bank0::Gpio25, gpio::FunctionSioOutput, gpio::PullDown>>>,
> = Shared::empty();

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let _ = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::Sio::new(pac.SIO);

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Turn LEDs off
    let _ = pins
        .led_red
        .into_push_pull_output_in_state(gpio::PinState::High);
    let _ = pins
        .led_green
        .into_push_pull_output_in_state(gpio::PinState::High);
    let _ = GLOBAL_BLUE_LED.replace(InvertedPin::new(
        pins.led_blue
            .into_push_pull_output_in_state(gpio::PinState::High),
    ));

    let i2c = I2C::i2c1(
        pac.I2C1,
        pins.sda.into_function(),
        pins.scl.into_function(),
        400.kHz(),
        &mut pac.RESETS,
        125_000_000.Hz(),
    );
    let mut display = Ssd1306::new(
        I2CDisplayInterface::new(i2c),
        DisplaySize128x64,
        DisplayRotation::Rotate270,
    )
    .into_terminal_mode();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    if display.init().is_ok() && display.clear().is_ok() {
        let _ = writeln!(display, "INIT OK");

        loop {
            for count in 0..u8::MAX {
                timer.delay_ms(250);
                let _ = writeln!(display, "{}TEST", count);
            }
        }
    } else {
        // Failed to setup display
        loop {
            cortex_m::asm::wfe();
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    if let Some(mut pin) = GLOBAL_BLUE_LED.take() {
        let _ = pin.set_high();
    }
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
