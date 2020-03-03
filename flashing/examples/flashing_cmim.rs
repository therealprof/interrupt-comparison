#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal::{gpio::*, prelude::*, stm32};

use cortex_m::{peripheral::syst::SystClkSource::Core, Peripherals};
use cortex_m_rt::{entry, exception};

use cmim::{
    Move,
    Context,
    Exception,
};

// Mutex protected structure for our shared GPIO pin
static GPIO: Move<flashing::LEDPIN, stm32::Interrupt> = Move::new_uninitialized(Context::Exception(Exception::SysTick));

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(mut cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(move |cs| {
            // Configure clock to 48 MHz (i.e. the maximum) and freeze it
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

            // Get access to individual pins in the GPIO port
            let gpioa = p.GPIOA.split(&mut rcc);

            // (Re-)configure the pin connected to our LED as output
            let led = gpioa.pa5.into_push_pull_output(cs);

            // Transfer GPIO into a shared structure
            GPIO.try_move(led).ok();

            // Set source for SysTick counter, here full operating frequency (== 48MHz)
            cp.SYST.set_clock_source(Core);

            // Set reload value, i.e. timer delay 48 MHz/4 Mcounts == 12Hz or 83ms
            cp.SYST.set_reload(4_000_000 - 1);

            // Start counting
            cp.SYST.enable_counter();

            // Enable interrupt generation
            cp.SYST.enable_interrupt();
        });
    }

    loop {
        continue;
    }
}

// Define an exception handler, i.e. function to call when exception occurs. Here, if our SysTick
// timer generates an exception the following handler will be called
#[exception]
fn SysTick() -> () {
    // Exception handler state variable
    static mut STATE: u8 = 0;

    GPIO.try_lock(|led| {
        // Check state variable, keep LED off most of the time and turn it on every 10th tick
        if *STATE < 10 {
            // Turn off the LED
            led.set_low().ok();

            // And now increment state variable
            *STATE += 1;
        } else {
            // Turn on the LED
            led.set_high().ok();

            // And set new state variable back to 0
            *STATE = 0;
        }
    }).ok();
}
