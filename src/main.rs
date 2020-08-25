#![no_main]
#![no_std]

use cortex_m::{asm::bkpt, asm::delay};
use cortex_m_rt::entry;
#[allow(unused_import)]
use rtt_target::{rprintln, rtt_init_print};
use ssd1306;

use crate::hal::{
    gpio::{gpioe::PE9, Output, PushPull},
    i2c,
    i2c::BlockingI2c,
    prelude::*,
    rcc::{HSEClock, HSEClockMode},
};
use stm32f7xx_hal as hal;

mod mcp4728;

use mcp4728::{Mcp4728, Mcp4728I2c};

#[rtfm::app(device=stm32f7xx_hal::pac, peripherals=true)]
const APP: () = {
    struct Resources {
        led1r: PE9<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        delay(1_600_000);
        rprintln!("Hello!");

        let peripherals = cx.device;

        let mut rcc = peripherals.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(108.mhz()).freeze();

        let gpioe = peripherals.GPIOE.split();
        let led1r = gpioe.pe9.into_push_pull_output();

        let gpiof = peripherals.GPIOF.split();
        // let i2c2 = BlockingI2c::i2c2(
        //     peripherals.I2C2,
        //     (
        //         gpiof.pf1.into_alternate_af4(),
        //         gpiof.pf0.into_alternate_af4(),
        //     ),
        //     i2c::Mode::Fast {
        //         frequency: 400.khz().into(),
        //     },
        //     clocks,
        //     &mut rcc.apb1,
        //     15000,
        // );

        // let interface = ssd1306::I2CDIBuilder::new().with_i2c_addr(0x3C).init(i2c2);
        // let mut disp: ssd1306::mode::TerminalMode<_> =
        //     ssd1306::Builder::new().connect(interface).into();
        // disp.init().unwrap();
        // disp.clear();
        // for c in "M-M-M-MultiMIDI".chars() {
        //     disp.print_char(c);
        // }
        // disp.set_position(0, 2);
        // for c in "It lives!".chars() {
        //     disp.print_char(c);
        // }

        // disp.set_position(0, 4);
        // for c in "#rustlang".chars() {
        //     disp.print_char(c);
        // }
        // disp.set_position(0, 5);
        // for c in "#embeddedrust".chars() {
        //     disp.print_char(c);
        // }
        // disp.set_position(0, 6);
        // for c in "Hello Penny! Meooo!".chars() {
        //     disp.print_char(c);
        // }
        // disp.set_position(0, 7);
        // for c in "Hello Emily!!".chars() {
        //     disp.print_char(c);
        // }
        // disp.flush();

        let gpiob = peripherals.GPIOB.split();

        let mut dac_i2c = Mcp4728I2c::new(
            &clocks,
            100.khz(),
            gpiob.pb10.into_open_drain_output(),
            gpiob.pb9.into_open_drain_output(),
        );
        let mut ldac1 = gpiof.pf8.into_push_pull_output();
        ldac1.set_high().unwrap();
        let mut ldac2 = gpiof.pf10.into_push_pull_output();
        ldac2.set_high().unwrap();
        let mut ldac3 = gpiof.pf12.into_push_pull_output();
        ldac3.set_high().unwrap();
        let mut ldac4 = gpiof.pf14.into_push_pull_output();
        ldac4.set_high().unwrap();

        let mut dac1 = Mcp4728::new(ldac1, 0x1, &mut dac_i2c).unwrap();
        let mut dac2 = Mcp4728::new(ldac2, 0x2, &mut dac_i2c).unwrap();
        let mut dac3 = Mcp4728::new(ldac3, 0x3, &mut dac_i2c).unwrap();
        let mut dac4 = Mcp4728::new(ldac4, 0x4, &mut dac_i2c).unwrap();

        dac1.set_channel(&mut dac_i2c, 0, 0x0).unwrap();
        dac1.set_channel(&mut dac_i2c, 1, 0x400).unwrap();
        dac1.set_channel(&mut dac_i2c, 2, 0x800).unwrap();
        dac1.set_channel(&mut dac_i2c, 3, 0xC00).unwrap();

        dac2.set_channel(&mut dac_i2c, 0, 0xFFF).unwrap();
        dac2.set_channel(&mut dac_i2c, 1, 0xC00).unwrap();
        dac2.set_channel(&mut dac_i2c, 2, 0x800).unwrap();
        dac2.set_channel(&mut dac_i2c, 3, 0x400).unwrap();

        dac3.set_channel(&mut dac_i2c, 0, 0x400).unwrap();
        dac3.set_channel(&mut dac_i2c, 1, 0x800).unwrap();
        dac3.set_channel(&mut dac_i2c, 2, 0xC00).unwrap();
        dac3.set_channel(&mut dac_i2c, 3, 0xFFF).unwrap();

        dac4.set_channel(&mut dac_i2c, 0, 0xFFF).unwrap();
        dac4.set_channel(&mut dac_i2c, 1, 0x800).unwrap();
        dac4.set_channel(&mut dac_i2c, 2, 0xC00).unwrap();
        dac4.set_channel(&mut dac_i2c, 3, 0x400).unwrap();

        init::LateResources { led1r }
    }

    #[idle(resources=[led1r])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            for i in 0..100000 {
                cx.resources.led1r.set_high().unwrap();
            }
            for i in 0..100000 {
                cx.resources.led1r.set_low().unwrap();
            }
        }
    }
};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", info);
    exit()
}

fn exit() -> ! {
    loop {
        bkpt()
    }
}
