
#![no_main]
#![no_std]

#[macro_use(entry)]
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32l4xx_hal as hal;
extern crate cortex_m;
extern crate embedded_hal;

use core::fmt::{self, Write};
use cortex_m::delay::{SysTickDelay, Delay};
use cortex_m::clock::{Clocks, Frequency, U32Ext};
use hal::rcc::{Clocks as RccClk, RccExt};
use hal::serial::{Serial, Tx};
use hal::gpio::GpioExt;
use hal::flash::FlashExt;
use hal::time::{Bps, MegaHertz};
use embedded_hal::digital::OutputPin;

struct DelayClock {
    rccclk: RccClk,
}

impl Clocks for DelayClock {
    fn get_external_syst_clock(&self) -> Frequency {
        (self.rccclk.hclk().0 / 8).hz()
    }

    fn get_core_clock(&self) -> Frequency {
        self.rccclk.sysclk().0.hz()
    }
}

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
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = hal::stm32::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);
    let mut gpiod = p.GPIOD.split(&mut rcc.ahb2);


    let clocks = rcc.cfgr.hclk(MegaHertz(4)).freeze(&mut flash.acr);

    let tx = gpiod.pd5.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
    let rx = gpiod.pd6.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
    let vcom = Serial::usart2(p.USART2, (tx, rx), Bps(115_200), clocks, &mut rcc.apb1r1);
    let (mut vcomtx, _rx) = vcom.split();

    let clocks = DelayClock { rccclk: clocks };
    let mut delay = SysTickDelay::new_external(cp.SYST, clocks);
    let mut now = delay.start();

    let mut pb2 = gpiob.pb2.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let mut buf = [0 as u8; 40];
    write!(Wrapper::new(&mut buf), "delay 1: {:?}\n\r", now.elapse()).unwrap();
    vcomtx.write_str(core::str::from_utf8(&buf).unwrap()).ok();
    write!(Wrapper::new(&mut buf), "delay 2: {:?}\n\r", now.elapse()).unwrap();
    vcomtx.write_str(core::str::from_utf8(&buf).unwrap()).ok();

    let mut delay = now.stop();

    loop {

        pb2.set_high();

        delay.delay(1.s());
        pb2.set_low();

        delay.delay(1.s());

    }
}
