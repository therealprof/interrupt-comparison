#![no_std]

use stm32f0xx_hal::gpio::*;

// A type definition for the GPIO pin to be used for our LED
pub type LEDPIN = gpioa::PA5<Output<PushPull>>;
