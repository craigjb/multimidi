use super::descriptors::*;
use rtt_target::rprintln;
use usb_device::{class_prelude::*, Result};

pub struct MidiClass<'a, B: UsbBus> {
    audio_control_interface: InterfaceNumber,
    midi_streaming_interface: InterfaceNumber,
    midi_in: EndpointOut<'a, B>,
}

impl<'a, B: UsbBus> MidiClass<'a, B> {
    pub fn new(alloc: &'a UsbBusAllocator<B>) -> Self {
        MidiClass {
            audio_control_interface: alloc.interface(),
            midi_streaming_interface: alloc.interface(),
            midi_in: alloc.bulk(64),
        }
    }

    pub fn read_packet(&self) -> Result<[u8; 4]> {
        let mut buf = [0; 4];
        self.midi_in.read(&mut buf)?;
        Ok(buf)
    }
}

impl<B: UsbBus> UsbClass<B> for MidiClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        rprintln!("Starting descriptors");

        rprintln!("AC descriptors");
        // Audio control
        writer.interface(self.audio_control_interface, AUDIO_CLASS, AUDIO_CONTROL, 0)?;
        writer.write(
            CS_INTERFACE,
            &[
                AC_HEADER, // audio control header
                0x00,
                0x01, // revision (little endian)
                0x09,
                0x00,                                 // total length -- just this header
                0x01,                                 // number of streaming interfaces = 1
                self.midi_streaming_interface.into(), // interface for MIDI streaming
            ],
        )?;

        rprintln!("MIDI descriptors");
        // MIDI streaming
        writer.interface(
            self.midi_streaming_interface,
            AUDIO_CLASS,
            MIDI_STREAMING,
            0,
        )?;

        let total_len = MS_HEADER_SIZE + MIDI_OUT_JACK_SIZE;
        writer.write(
            CS_INTERFACE,
            &[
                MS_HEADER, // MIDI Header
                0x00,
                0x01, // revision (little endian)
                (total_len & 0xFF) as u8,
                ((total_len & 0xFF00) >> 8) as u8, // total length (little endian)
            ],
        )?;

        writer.write(
            CS_INTERFACE,
            &[
                MIDI_IN_JACK,
                EMBEDDED,
                0x01, // jack id
                0x00, // unused
            ],
        )?;

        writer.endpoint(&self.midi_in)?;
        writer.write(
            CS_ENDPOINT,
            &[
                MS_GENERAL, // MIDI general endpoint
                0x01,       // number of embedded jacks
                0x01,       // id of embedded jack
            ],
        )?;

        rprintln!("Done with descriptors");
        Ok(())
    }
}
