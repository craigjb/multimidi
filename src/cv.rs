use crate::hal::{
    gpio::{
        gpiob::{PB10, PB9},
        gpiod::{PD0, PD1, PD2, PD3, PD4, PD5, PD6, PD7},
        gpiof::{PF10, PF12, PF14, PF8},
        Floating, Input, OpenDrain, Output, PushPull,
    },
    rcc::Clocks,
    time::U32Ext,
};
use crate::mcp4728::{Mcp4728, Mcp4728Error, Mcp4728I2c};
use embedded_hal::digital::v2::{InputPin, OutputPin};

type SCL = PB10<Output<OpenDrain>>;
type SDA = PB9<Output<OpenDrain>>;

pub struct CvPanel {
    i2c: Mcp4728I2c<SCL, SDA>,
    dacs: [Mcp4728<SCL, SDA>; 4],
}

impl CvPanel {
    pub fn new(
        clocks: &Clocks,
        mut ldac1: PF8<Output<PushPull>>,
        mut ldac2: PF10<Output<PushPull>>,
        mut ldac3: PF12<Output<PushPull>>,
        mut ldac4: PF14<Output<PushPull>>,
        scl: PB10<Output<OpenDrain>>,
        sda: PB9<Output<OpenDrain>>,
    ) -> Self {
        let mut i2c = Mcp4728I2c::new(&clocks, 100.khz(), scl, sda);
        ldac1.set_high().unwrap();
        ldac2.set_high().unwrap();
        ldac3.set_high().unwrap();
        ldac4.set_high().unwrap();
        let dac1 = Mcp4728::new(ldac1, 0x1, &mut i2c).unwrap();
        let dac2 = Mcp4728::new(ldac2, 0x2, &mut i2c).unwrap();
        let dac3 = Mcp4728::new(ldac3, 0x3, &mut i2c).unwrap();
        let dac4 = Mcp4728::new(ldac4, 0x4, &mut i2c).unwrap();
        Self {
            i2c,
            dacs: [dac1, dac2, dac3, dac4],
        }
    }

    pub fn gate<'a>(&'a mut self, voice: usize) -> Cv<'a> {
        assert!(voice <= 3);
        Cv::<'a> {
            panel: self,
            dac: voice,
            channel: 3,
        }
    }

    pub fn pitch<'a>(&'a mut self, voice: usize) -> Cv<'a> {
        assert!(voice <= 3);
        Cv::<'a> {
            panel: self,
            dac: voice,
            channel: 2,
        }
    }

    pub fn aux1<'a>(&'a mut self, voice: usize) -> Cv<'a> {
        assert!(voice <= 3);
        Cv::<'a> {
            panel: self,
            dac: voice,
            channel: 1,
        }
    }

    pub fn aux2<'a>(&'a mut self, voice: usize) -> Cv<'a> {
        assert!(voice <= 3);
        Cv::<'a> {
            panel: self,
            dac: voice,
            channel: 0,
        }
    }
}

pub struct Cv<'a> {
    panel: &'a mut CvPanel,
    dac: usize,
    channel: u8,
}

impl<'a> Cv<'a> {
    pub fn set(&mut self, value: u16) -> Result<(), Mcp4728Error> {
        self.panel.dacs[self.dac].set_channel(&mut self.panel.i2c, self.channel, value)
    }
}
