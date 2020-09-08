#![no_main]
#![no_std]

use cortex_m::{asm::bkpt, asm::delay};
use rtt_target::{rprintln, rtt_init_print};

use crate::hal::{
    gpio::{gpioe::PE9, Output, PushPull},
    prelude::*,
    rcc::{HSEClock, HSEClockMode},
};
use stm32f7xx_hal as hal;

mod cv;
mod encoder;
mod mcp4728;
mod midi;
mod usb_fs;

use cv::CvPanel;
use encoder::Encoder;
use midi::device::MidiClass;
use usb_device::prelude::*;
use usb_fs::{UsbBus, UsbBusType};

#[rtic::app(device=stm32f7xx_hal::pac, peripherals=true)]
const APP: () = {
    struct Resources {
        led1r: PE9<Output<PushPull>>,
        encoder: Encoder,
        cv_panel: CvPanel,
        usb_device: usb_device::device::UsbDevice<'static, UsbBusType>,
        midi_device: MidiClass<'static, UsbBusType>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        delay(1_600_000);
        rprintln!("Hello!");

        let peripherals = cx.device;

        let rcc = peripherals.RCC.constrain();
        let clocks = rcc
            .cfgr
            .hse(HSEClock::new(12.mhz(), HSEClockMode::Oscillator))
            .sysclk(192.mhz())
            .usb(true)
            .freeze();

        let gpioe = peripherals.GPIOE.split();
        let led1r = gpioe.pe9.into_push_pull_output();

        let gpiof = peripherals.GPIOF.split();
        let gpiob = peripherals.GPIOB.split();

        let cv_panel = CvPanel::new(
            &clocks,
            gpiof.pf8.into_push_pull_output(),
            gpiof.pf10.into_push_pull_output(),
            gpiof.pf12.into_push_pull_output(),
            gpiof.pf14.into_push_pull_output(),
            gpiob.pb10.into_open_drain_output(),
            gpiob.pb9.into_open_drain_output(),
        );

        let gpioc = peripherals.GPIOC.split();
        let encoder = Encoder::new(
            peripherals.TIM3,
            gpioc.pc6.into_alternate_af2(),
            gpioc.pc7.into_alternate_af2(),
            gpioc.pc5.into_floating_input(),
        );

        let gpioa = peripherals.GPIOA.split();

        static mut EP_MEM: [u32; 1024] = [0; 1024];
        static mut USB_BUS: Option<
            usb_device::bus::UsbBusAllocator<synopsys_usb_otg::UsbBus<usb_fs::USB>>,
        > = None;
        unsafe {
            USB_BUS = Some(UsbBus::new(
                usb_fs::USB {
                    usb_global: peripherals.OTG_FS_GLOBAL,
                    usb_device: peripherals.OTG_FS_DEVICE,
                    usb_pwrclk: peripherals.OTG_FS_PWRCLK,
                    pin_dm: gpioa.pa11.into_alternate_af10(),
                    pin_dp: gpioa.pa12.into_alternate_af10(),
                    hclk: clocks.hclk(),
                },
                &mut EP_MEM,
            ));
        }

        let midi_device = MidiClass::new(unsafe { USB_BUS.as_ref().unwrap() });

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { USB_BUS.as_ref().unwrap() },
            UsbVidPid(0x16c0, 0x27dd),
        )
        .manufacturer("craigjb.com")
        .product("M-M-M-MultiMIDI")
        .serial_number("0.1.1")
        .build();

        init::LateResources {
            led1r,
            encoder,
            cv_panel,
            usb_device: usb_dev,
            midi_device,
        }
    }

    // #[task(binds=OTG_FS, priority=3, resources=[usb_device, midi_device])]
    // fn interrupt_usb(cx: interrupt_usb::Context) {
    // }

    #[idle(resources=[led1r, encoder, cv_panel, usb_device, midi_device])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            if cx
                .resources
                .usb_device
                .poll(&mut [cx.resources.midi_device])
            {
                if let Ok(packet) = cx.resources.midi_device.read_packet() {
                    if packet[1] & 0xf0 == 0x90 {
                        let offset = (packet[2] - 24) as u16;
                        cx.resources
                            .cv_panel
                            .pitch(0)
                            .set((800 + (offset * 34)).min(4095))
                            .unwrap();
                        rprintln!("Note on: {}", offset);
                    } else if packet[1] & 0xf0 == 0x80 {
                        cx.resources.cv_panel.pitch(0).set(0).unwrap();
                        let offset = (packet[2] - 24) as u16;
                        rprintln!("Note off: {}", offset);
                    }
                }
            }

            // rprintln!("Idle...");
            // delay(1000000);
            // }
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
