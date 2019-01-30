//! Test the serial interface
//!
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
// #[macro_use(block)]
// extern crate nb;
extern crate panic_semihosting;

extern crate stm32l4xx_hal as hal;
extern crate mfxstm32l152 as mfx;
// #[macro_use(block)]
// extern crate nb;

use cortex_m::asm;
use crate::rt::ExceptionFrame;

use core::fmt::{self, Write};

extern crate discovery32l476 as disco;

struct Wrapper<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> Wrapper<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Wrapper {
            buf: buf,
            offset: 0,
        }
    }
}

impl<'a> fmt::Write for Wrapper<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();

        // Skip over already-copied data
        let remainder = &mut self.buf[self.offset..];
        // Check if there is space remaining (return error instead of panicking)
        if remainder.len() < bytes.len() { return Err(core::fmt::Error); }
        // Make the two slices the same length
        let remainder = &mut remainder[..bytes.len()];
        // Copy
        remainder.copy_from_slice(bytes);

        // Update offset to avoid overwriting
        self.offset += bytes.len();

        Ok(())
    }
}

#[entry]
fn main() -> ! {

    let mut board = disco::Board::freeze();

    board.idd_init().unwrap();
    let idd = board.idd_measure().unwrap();


    let mut buf = [0 as u8; 40];
    write!(Wrapper::new(&mut buf), "\n\rIDD: {}\n\r\0", idd).unwrap();

    board.vcomtx.write_str(core::str::from_utf8(&buf).unwrap()).ok();

    // if all goes well you should reach this breakpoint
    asm::bkpt();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
