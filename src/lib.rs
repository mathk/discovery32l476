#![no_std]

extern crate stm32l4xx_hal as hal;

use crate::hal::prelude::*;
use crate::hal::gpio::{gpioa, gpiod};
use crate::hal::serial::Serial;

use crate::hal::gpio::gpiod::{PD5, PD6};
use crate::hal::gpio::{AF7, Alternate, Input, Floating};
use hal::stm32::USART2;

type VcomPins = (PD5<Alternate<AF7, Input<Floating>>>, PD6<Alternate<AF7, Input<Floating>>>);

pub struct Board {
    gpioa: gpioa::Parts,
    vcom: Serial<USART2, VcomPins>,
}


pub trait IddMeasure<E> {
    fn init(&mut self) -> Result<(), E>;
    fn read_idd() -> Result<u32, E>;
}

impl Board {

    pub fn freeze() -> Board {
        // let cp = cortex_m::Peripherals::take().unwrap();
        let p = hal::stm32::Peripherals::take().unwrap();

        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let gpioa = p.GPIOA.split(&mut rcc.ahb2);
        let mut gpiod = p.GPIOD.split(&mut rcc.ahb2);
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        // VCOM gpio
        let tx = gpiod.pd5.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
        let rx = gpiod.pd6.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
        let vcom = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb1r1);
        Board {
            gpioa,
            vcom,
        }
    }
}
