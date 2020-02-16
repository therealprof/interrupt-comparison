#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal::{gpio::*, prelude::*, stm32};

use cortex_m::{peripheral::syst::SystClkSource::Core, Peripherals};
use cortex_m_rt::{entry, exception};
use irq::{handler, scope, scoped_interrupts};

// Hook `SysTick` using the `#[exception]` attribute
scoped_interrupts! {
    enum Exception {
        SysTick,
    }

    use #[exception];
}

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(mut cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let mut led = cortex_m::interrupt::free(move |cs| {
            // Configure clock to 48 MHz (i.e. the maximum) and freeze it
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

            // Get access to individual pins in the GPIO port
            let gpioa = p.GPIOA.split(&mut rcc);

            // (Re-)configure the pin connected to our LED as output
            let led = gpioa.pa5.into_push_pull_output(cs);

            // Set source for SysTick counter, here full operating frequency (== 48MHz)
            cp.SYST.set_clock_source(Core);

            // Set reload value, i.e. timer delay 48 MHz/4 Mcounts == 12Hz or 83ms
            cp.SYST.set_reload(4_000_000 - 1);

            // Start counting
            cp.SYST.enable_counter();

            // Enable interrupt generation
            cp.SYST.enable_interrupt();

            led
        });

        // State variable
        let mut state: u8 = 0;

        handler!(
            systick = || {
                // Check state variable, keep LED off most of the time and turn it on every 10th tick
                if state < 10 {
                    // Turn off the LED
                    led.set_low().ok();

                    // And now increment state variable
                    state += 1;
                } else {
                    // Turn on the LED
                    led.set_high().ok();

                    // And set new state variable back to 0
                    state = 0;
                }
            }
        );

        // Create a scope and register the handlers
        scope(|scope| {
            scope.register(Exception::SysTick, systick);

            loop {
                continue;
            }
        });
    }

    loop {
        continue;
    }
}
