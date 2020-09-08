use crate::hal::{
    gpio::{
        gpioc::{PC5, PC6, PC7},
        Alternate, Floating, Input, AF2,
    },
    pac::{RCC, TIM3},
};
use embedded_hal::digital::v2::InputPin;

pub struct Encoder {
    timer: TIM3,
    select: PC5<Input<Floating>>,
}

impl Encoder {
    pub fn new(
        tim3: TIM3,
        _pc6: PC6<Alternate<AF2>>,
        _pc7: PC7<Alternate<AF2>>,
        pc5: PC5<Input<Floating>>,
    ) -> Self {
        let rcc = unsafe { &(*RCC::ptr()) };
        rcc.apb1enr.modify(|_, w| w.tim3en().set_bit());

        // Configure TxC1 and TxC2 as captures
        tim3.ccmr1_input().write(|w| w.cc1s().ti1().cc2s().ti2());

        // enable and configure to capture on rising edge
        tim3.ccer.write(|w| {
            w.cc1e()
                .set_bit()
                .cc1p()
                .clear_bit()
                .cc2e()
                .set_bit()
                .cc2p()
                .clear_bit()
        });

        // configure as quadrature encoder
        tim3.smcr.write(|w| w.sms().bits(0b001));

        tim3.arr.write(|w| w.arr().bits(0xFFFF));
        tim3.cnt.write(|w| w.cnt().bits(0x8000));
        tim3.cr1.write(|w| w.cen().set_bit());

        Self {
            timer: tim3,
            select: pc5,
        }
    }

    pub fn reset(&mut self) {
        self.timer.cnt.write(|w| w.cnt().bits(0x8000));
    }

    pub fn count(&self) -> u32 {
        self.timer.cnt.read().cnt().bits() as u32
    }

    pub fn select_pressed(&self) -> bool {
        self.select.is_low().unwrap()
    }
}
