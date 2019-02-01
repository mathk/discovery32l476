#![no_std]

extern crate stm32l4xx_hal as hal;
extern crate mfxstm32l152 as mfx;

use core::marker::PhantomData;
use hal::prelude::*;
use hal::serial::{Serial, Tx};
use hal::delay::Delay;
use hal::i2c::{I2c, Error as I2cError};
use hal::rcc::{Clocks, Rcc};

use hal::gpio::{gpioa, gpioa::{PA0, PA4}, gpiob::{PB10, PB11}};
use hal::gpio::{Input, Output, Floating, PushPull, OpenDrain};
use hal::gpio::{AF4, Alternate};

use hal::stm32::USART2;
use hal::stm32::I2C2;

use mfx::{MFX, DelayUnit, NbShunt, Ampere};

type MfxI2c = I2c<I2C2, (PB10<Alternate<AF4, Output<OpenDrain>>>, PB11<Alternate<AF4, Output<OpenDrain>>>)>;

static DISCOVERY_IDD_AMPLI_GAIN : u16 =  4967;   // value is gain * 100
// On rev B and A
// static DISCOVERY_IDD_AMPLI_GAIN : u16 =  4990;     /*!< value is gain * 100 */




pub struct RunMode;

pub struct Board<Mode> {
    _phantom: PhantomData<Mode>,
    pub mfx: Option<MFX<MfxI2c, PA4<Output<PushPull>>, Delay>>,
}

impl<T> Board<T> {

    pub fn constrain() -> Board<RunMode> {
        Board {
            _phantom: PhantomData,
            mfx: None,
        }
    }
}

impl Board<RunMode> {

    pub fn init_mfx(&mut self, mut mfx: MFX<MfxI2c, PA4<Output<PushPull>>, Delay>) -> Result<(), I2cError> {
        mfx.set_idd_ctrl(false, false, NbShunt::SHUNT_NB_4).unwrap();
        mfx.set_idd_gain(DISCOVERY_IDD_AMPLI_GAIN).unwrap();
        mfx.set_idd_vdd_min(2000).unwrap(); // In milivolt
        mfx.set_idd_pre_delay(DelayUnit::TIME_20_MS, 0xF).unwrap(); // Max delay

        // The shunt resistor is in the user manual.
        // Delay is pick from the stmcubel4 driver
        mfx.set_idd_shunt0(1000, 149)?;
        mfx.set_idd_shunt1(24, 149)?;
        mfx.set_idd_shunt2(620, 149)?;
        mfx.set_idd_shunt3(0, 0)?;
        mfx.set_idd_shunt4(10000, 255)?;
        mfx.set_idd_nb_measurment(10)?;
        mfx.set_idd_meas_delta_delay(DelayUnit::TIME_5_MS, 10)?;

        self.mfx = Some(mfx);
        Ok(())
    }
}


/*
pub struct Board<Mode> {
    phantom: PhantomData<Mode>,
    pa0: PA0<Input<Floating>>,
    pub vcomtx: Tx<USART2>,
    mfx: MFX<MfxI2c, PA4<Output<PushPull>>, Delay>,
}

pub struct PortA {
    otyper: gpioa::OTYPER,
    moder: gpioa::MODER,
    pa0: PA0<Input<Floating>>,
}


pub struct RunMode;
pub trait WithVcom {
    fn init_vcom(&mut self,  ) -> Serial;
};


impl<T> Board<T>
where
    T: WithVcom
{
    pub fn initVcom(&mut self) {

    }
}

impl<T> Board<T>
where
    T: RunMode + WithVcom
{

    pub fn freeze() -> Board<T> {
        let cp = cortex_m::Peripherals::take().unwrap();
        let p = hal::stm32::Peripherals::take().unwrap();

        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);
        let mut gpiod = p.GPIOD.split(&mut rcc.ahb2);
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        // VCOM gpio
        let tx = gpiod.pd5.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
        let rx = gpiod.pd6.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
        let vcom = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb1r1);
        let (vcomtx, _rx) = vcom.split();

        let mut scl = gpiob.pb10.into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
        scl.internal_pull_up(&mut gpiob.pupdr, true);
        let scl = scl.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

        let mut sda = gpiob.pb11.into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
        sda.internal_pull_up(&mut gpiob.pupdr, true);
        let sda = sda.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

        let wakup = gpioa.pa4.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        let i2c = I2c::i2c2(p.I2C2, (scl, sda), 100.khz(), clocks, &mut rcc.apb1r1);
        let timer = Delay::new(cp.SYST, clocks);
        let mfx = MFX::new(i2c, wakup, timer, 0x84).unwrap();

        let b = Board {
            pa0: gpioa.pa0,
            vcomtx,
            mfx,
        };
        b.initVcom();
        b

    }


    pub fn idd_init(&mut self) -> Result<(), I2cError>{
        self.mfx.set_idd_ctrl(false, false, NbShunt::SHUNT_NB_4).unwrap();
        self.mfx.set_idd_gain(DISCOVERY_IDD_AMPLI_GAIN).unwrap();
        self.mfx.set_idd_vdd_min(2000).unwrap(); // In milivolt
        self.mfx.set_idd_pre_delay(DelayUnit::TIME_20_MS, 0xF).unwrap(); // Max delay

        // The shunt resistor is in the user manual.
        // Delay is pick from the stmcubel4 driver
        self.mfx.set_idd_shunt0(1000, 149)?;
        self.mfx.set_idd_shunt1(24, 149)?;
        self.mfx.set_idd_shunt2(620, 149)?;
        self.mfx.set_idd_shunt3(0, 0)?;
        self.mfx.set_idd_shunt4(10000, 255)?;
        self.mfx.set_idd_nb_measurment(10)?;
        self.mfx.set_idd_meas_delta_delay(DelayUnit::TIME_5_MS, 10)
    }

    pub fn idd_measure(&mut self) -> Result<Ampere, I2cError> {
        self.mfx.idd_start()?;
        self.mfx.idd_get_value()
    }
}
*/
