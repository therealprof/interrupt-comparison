#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal::{gpio::*, prelude::*, stm32};

use cortex_m::peripheral::syst::SystClkSource::Core;

use rtfm::app;

#[app(device = crate::stm32, peripherals = true)]
const APP: () = {
    // Late resources
    struct Resources {
        led: flashing::LEDPIN,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Cortex-M peripherals

        let mut core = cx.core;

        // Device specific peripherals
        let mut device = cx.device;

        // Configure clock to 48 MHz (i.e. the maximum) and freeze it
        let mut rcc = device
            .RCC
            .configure()
            .sysclk(48.mhz())
            .freeze(&mut device.FLASH);

        // Get access to individual pins in the GPIO port
        let gpioa = device.GPIOA.split(&mut rcc);

        let led = cortex_m::interrupt::free(move |cs| {
            // (Re-)configure the pin connected to our LED as output
            gpioa.pa5.into_push_pull_output(cs)
        });

        // Set source for SysTick counter, here full operating frequency (== 48MHz)
        core.SYST.set_clock_source(Core);

        // Set reload value, i.e. timer delay 48 MHz/4 Mcounts == 12Hz or 83ms
        core.SYST.set_reload(4_000_000 - 1);

        // Start counting
        core.SYST.enable_counter();

        // Enable interrupt generation
        core.SYST.enable_interrupt();

        init::LateResources { led }
    }

    // Define an exception handler, i.e. function to call when exception occurs. Here, if our SysTick
    // timer generates an exception the following handler will be called
    #[task(binds = SysTick, priority = 1, resources = [led])]
    fn systick(c: systick::Context) {
        // Exception handler state variable
        static mut STATE: u8 = 0;

        // If LED pin was moved into the exception handler, just use it
        // Check state variable, keep LED off most of the time and turn it on every 10th tick
        if *STATE < 10 {
            // Turn off the LED
            c.resources.led.set_low().ok();

            // And now increment state variable
            *STATE += 1;
        } else {
            // Turn on the LED
            c.resources.led.set_high().ok();

            // And set new state variable back to 0
            *STATE = 0;
        }
    }
};
