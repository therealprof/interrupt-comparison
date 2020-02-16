#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal::{delay::Delay, prelude::*, stm32};

use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let mut state: u8 = 0;

        let (mut led, mut delay) = cortex_m::interrupt::free(|cs| {
            // Configure clock to 48 MHz (i.e. the maximum) and freeze it
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

            // Obtain resources from GPIO port A
            let gpioa = p.GPIOA.split(&mut rcc);

            // (Re-)configure PA5 as output
            let led = gpioa.pa5.into_push_pull_output(cs);

            // Get delay provider
            let delay = Delay::new(cp.SYST, &rcc);

            (led, delay)
        });

        loop {
            if state < 10 {
                // Turn off the LED
                led.set_low().ok();
                state += 1;
            } else {
                // Turn on the LED
                led.set_high().ok();
                state = 0;
            }
            delay.delay_ms(100_u16);
        }
    }

    loop {
        continue;
    }
}
