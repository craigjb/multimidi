use crate::hal::{rcc::Clocks, time::Hertz};
use core::fmt::Debug;
use core::marker::PhantomData;
use cortex_m::asm::delay;
use embedded_hal::digital::v2::{InputPin, OutputPin};

const GENERAL_CALL_ADDR: u8 = 0x0;
const DEVICE_CODE: u8 = 0x60;

pub struct Mcp4728<P, SCL, SDA>
where
    P: OutputPin,
    <P as embedded_hal::digital::v2::OutputPin>::Error: Debug,
    SCL: OutputPin,
    SCL::Error: Debug,
    SDA: OutputPin + InputPin,
    <SDA as embedded_hal::digital::v2::OutputPin>::Error: Debug,
    <SDA as embedded_hal::digital::v2::InputPin>::Error: Debug,
{
    ldac: P,
    address: u8,
    v0: u16,
    v1: u16,
    v2: u16,
    v3: u16,
    _i2c: PhantomData<Mcp4728I2c<SCL, SDA>>,
}

impl<P: OutputPin, SCL, SDA> Mcp4728<P, SCL, SDA>
where
    P: OutputPin,
    <P as embedded_hal::digital::v2::OutputPin>::Error: Debug,
    SCL: OutputPin,
    SCL::Error: Debug,
    SDA: OutputPin + InputPin,
    <SDA as embedded_hal::digital::v2::OutputPin>::Error: Debug,
    <SDA as embedded_hal::digital::v2::InputPin>::Error: Debug,
{
    pub fn new(
        mut ldac: P,
        address: u8,
        i2c: &mut Mcp4728I2c<SCL, SDA>,
    ) -> Result<Self, Mcp4728Error> {
        assert!(address <= 0x7);
        ldac.set_high().unwrap();

        let current_addr = Self::read_address(&mut ldac, i2c)?;
        let new_addr = DEVICE_CODE | address;
        if current_addr != new_addr {
            Self::write_address(&mut ldac, current_addr, new_addr, i2c)?;
        }

        Self::write_channel(new_addr, 0, 0x0, i2c)?;
        Self::write_channel(new_addr, 1, 0x0, i2c)?;
        Self::write_channel(new_addr, 2, 0x0, i2c)?;
        Self::write_channel(new_addr, 3, 0x0, i2c)?;
        ldac.set_low().unwrap();
        delay(i2c.full_delay);
        ldac.set_high().unwrap();
        Ok(Self {
            ldac,
            address: new_addr,
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            _i2c: PhantomData,
        })
    }

    pub fn set_channel(
        &mut self,
        i2c: &mut Mcp4728I2c<SCL, SDA>,
        channel: u8,
        value: u16,
    ) -> Result<(), Mcp4728Error> {
        assert!(channel <= 3);
        assert!(value < 4096);
        Self::write_channel(self.address, channel, value, i2c)
    }

    fn read_address(ldac: &mut P, i2c: &mut Mcp4728I2c<SCL, SDA>) -> Result<u8, Mcp4728Error> {
        ldac.set_high().unwrap();
        i2c.start(GENERAL_CALL_ADDR, false)?;
        i2c.write_byte_ldac(0x0C, ldac);
        i2c.check_ack()?;
        i2c.start(DEVICE_CODE, true)?;
        let data = i2c.read_byte(false);
        // i2c.stop();
        ldac.set_high().unwrap();
        let addr1 = (data & 0xE0) >> 5;
        let addr2 = (data & 0x0E) >> 1;
        let check = data & 0x11;
        if addr1 != addr2 || check != 0x10 {
            Err(Mcp4728Error::AddressMismatch)
        } else {
            Ok(addr1 | DEVICE_CODE)
        }
    }

    fn write_address(
        ldac: &mut P,
        current_addr: u8,
        new_addr: u8,
        i2c: &mut Mcp4728I2c<SCL, SDA>,
    ) -> Result<(), Mcp4728Error> {
        ldac.set_high().unwrap();
        i2c.start(current_addr, false)?;
        i2c.write_byte_ldac(0x61 | ((current_addr & 0x7) << 2), ldac);
        i2c.check_ack()?;
        i2c.write_byte(0x62 | ((new_addr & 0x7) << 2));
        i2c.check_ack()?;
        i2c.write_byte(0x63 | ((new_addr & 0x7) << 2));
        i2c.check_ack()?;
        i2c.stop();
        ldac.set_high().unwrap();
        Ok(())
    }

    fn write_channel(
        addr: u8,
        channel: u8,
        value: u16,
        i2c: &mut Mcp4728I2c<SCL, SDA>,
    ) -> Result<(), Mcp4728Error> {
        assert!(channel <= 3);
        i2c.start(addr, false)?;
        i2c.write_byte(0x40 | (channel << 1));
        i2c.check_ack()?;
        i2c.write_byte(((value & 0xF00) >> 8) as u8 | 0x80);
        i2c.check_ack()?;
        i2c.write_byte((value & 0xFF) as u8);
        i2c.check_ack()?;
        i2c.stop();
        Ok(())
    }
}

#[derive(Debug)]
pub enum Mcp4728Error {
    NoAck,
    AddressMismatch,
}

pub struct Mcp4728I2c<SCL, SDA>
where
    SCL: OutputPin,
    SCL::Error: Debug,
    SDA: OutputPin + InputPin,
    <SDA as embedded_hal::digital::v2::OutputPin>::Error: Debug,
    <SDA as embedded_hal::digital::v2::InputPin>::Error: Debug,
{
    scl: SCL,
    sda: SDA,
    half_delay: u32,
    full_delay: u32,
}

impl<SCL, SDA> Mcp4728I2c<SCL, SDA>
where
    SCL: OutputPin,
    SCL::Error: Debug,
    SDA: OutputPin + InputPin,
    <SDA as embedded_hal::digital::v2::OutputPin>::Error: Debug,
    <SDA as embedded_hal::digital::v2::InputPin>::Error: Debug,
{
    pub fn new<F: Into<Hertz>>(clocks: &Clocks, freq: F, mut scl: SCL, mut sda: SDA) -> Self {
        let cycles = clocks.sysclk().0 as f32 / freq.into().0 as f32 / 20.0;
        scl.set_high().unwrap();
        sda.set_high().unwrap();
        Self {
            scl,
            sda,
            half_delay: (cycles / 4.0) as u32,
            full_delay: (cycles / 2.0) as u32,
        }
    }

    pub fn start(&mut self, addr: u8, read: bool) -> Result<(), Mcp4728Error> {
        let data = (addr << 1) | if read { 1 } else { 0 };

        // start condition
        self.scl.set_high().unwrap();
        self.sda.set_high().unwrap();
        delay(self.full_delay);
        self.sda.set_low().unwrap();
        delay(self.full_delay);
        self.scl.set_low().unwrap();
        delay(self.half_delay);

        // addr + rw
        self.write_byte(data);

        // check for ack
        self.check_ack()
    }

    fn write_byte_ldac<P: OutputPin>(&mut self, data: u8, ldac: &mut P)
    where
        <P as embedded_hal::digital::v2::OutputPin>::Error: Debug,
    {
        for offset in (0..8).rev() {
            if data & (1 << offset) != 0 {
                self.sda.set_high().unwrap();
            } else {
                self.sda.set_low().unwrap();
            }
            delay(self.half_delay);
            self.scl.set_high().unwrap();
            delay(self.full_delay);
            self.scl.set_low().unwrap();
            delay(self.half_delay);
            self.sda.set_low().unwrap();
            if offset == 0 {
                ldac.set_low().unwrap();
            }
        }
    }

    fn write_byte(&mut self, data: u8) {
        for offset in (0..8).rev() {
            if data & (1 << offset) != 0 {
                self.sda.set_high().unwrap();
            } else {
                self.sda.set_low().unwrap();
            }
            delay(self.half_delay);
            self.scl.set_high().unwrap();
            delay(self.full_delay);
            self.scl.set_low().unwrap();
            delay(self.half_delay);
            self.sda.set_low().unwrap();
        }
    }

    fn read_byte(&mut self, should_ack: bool) -> u8 {
        let mut byte: u8 = 0;

        self.sda.set_high().unwrap();

        for bit_offset in (0..8).rev() {
            self.scl.set_high().unwrap();
            delay(self.full_delay);

            if self.sda.is_high().unwrap() {
                byte |= 1 << bit_offset;
            }

            self.scl.set_low().unwrap();
            delay(self.full_delay);
        }

        if should_ack {
            self.sda.set_low().unwrap();
        } else {
            self.sda.set_high().unwrap();
        }

        self.scl.set_high().unwrap();
        delay(self.full_delay);

        self.scl.set_low().unwrap();
        self.sda.set_low().unwrap();
        delay(self.full_delay);

        byte
    }

    fn check_ack(&mut self) -> Result<(), Mcp4728Error> {
        if !self.is_ack() {
            return Err(Mcp4728Error::NoAck);
        } else {
            Ok(())
        }
    }

    fn is_ack(&mut self) -> bool {
        self.sda.set_high().unwrap();
        self.scl.set_high().unwrap();
        delay(self.full_delay);
        let ack = self.sda.is_low().unwrap();
        self.scl.set_low().unwrap();
        self.sda.set_high().unwrap();
        delay(self.full_delay);
        ack
    }

    fn stop(&mut self) {
        self.scl.set_high().unwrap();
        delay(self.full_delay);
        self.sda.set_high().unwrap();
        delay(self.full_delay);
    }
}
