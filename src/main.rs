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
mod usb_fs;

use cv::CvPanel;
use encoder::Encoder;
use usb_device::prelude::*;
use usb_fs::{UsbBus, UsbBusType};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[rtic::app(device=stm32f7xx_hal::pac, peripherals=true)]
const APP: () = {
    struct Resources {
        led1r: PE9<Output<PushPull>>,
        encoder: Encoder,
        cv_panel: CvPanel,
        usb_device: usb_device::device::UsbDevice<'static, UsbBusType>,
        serial: SerialPort<'static, UsbBusType>,
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

        let serial = SerialPort::new(unsafe { USB_BUS.as_ref().unwrap() });

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { USB_BUS.as_ref().unwrap() },
            UsbVidPid(0x16c0, 0x27dd),
        )
        .manufacturer("craigjb.com")
        .product("M-M-M-MultiMIDI")
        .serial_number("0.1.1")
        .device_class(USB_CLASS_CDC)
        .build();

        init::LateResources {
            led1r,
            encoder,
            cv_panel,
            usb_device: usb_dev,
            serial,
        }
    }

    #[task(binds=OTG_FS, priority=3, resources=[usb_device, serial])]
    fn interrupt_usb(cx: interrupt_usb::Context) {
        let mut buf = [0u8; 64];
        if !cx.resources.usb_device.poll(&mut [cx.resources.serial]) {
            return;
        }

        let read_count = match cx.resources.serial.read(&mut buf[..]) {
            Ok(count) => count,
            _ => 0,
        };

        if read_count > 0 {
            cx.resources.serial.write(&buf[0..read_count]).unwrap();
        }
    }

    #[idle(resources=[led1r, encoder, cv_panel])]
    fn idle(cx: idle::Context) -> ! {
        loop {}
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
