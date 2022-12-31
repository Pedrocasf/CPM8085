#![no_std]
#![no_main]
#![feature(once_cell)]
use core::{arch::global_asm, cell::{RefCell, OnceCell}, convert::Infallible};

use arduino_hal::{
    clock::MHz16,
    port::{
        mode::{
            Input,
            Output
        },
        Pin
    },
    prelude::*,
    hal::{
        port::{
            PE0,
            PE1
        },
        Usart
    }
};
use arduino_hal::pac::USART0;
use panic_halt as _;
extern crate symlog;
use symlog::{error, trace, log};
use embedded_hal::serial::Read;
#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    trace!("Hello from Arduino!\r{}",nope);

    loop {
        // Read a byte from the serial connection
        let b = nb::block!(serial.read()).void_unwrap();

        // Answer
        ufmt::uwriteln!(&mut serial, "Got {}!\r", b).void_unwrap();
    }
}