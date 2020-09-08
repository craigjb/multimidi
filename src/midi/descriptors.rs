// USB Class
pub const AUDIO_CLASS: u8 = 0x01;
pub const CS_INTERFACE: u8 = 0x24;
pub const CS_ENDPOINT: u8 = 0x25;

// USB Subclass
pub const AUDIO_CONTROL: u8 = 0x01;
pub const MIDI_STREAMING: u8 = 0x03;

// Audio subclass
pub const AC_HEADER: u8 = 0x01;

// MIDI interface descriptor subtypes
pub const MS_HEADER: u8 = 0x01;
pub const MS_HEADER_SIZE: usize = 7;
pub const MIDI_IN_JACK: u8 = 0x02;
pub const MIDI_IN_JACK_SIZE: usize = 6;
pub const MIDI_OUT_JACK: u8 = 0x03;
pub const MIDI_OUT_JACK_SIZE: usize = 9;
pub const ELEMENT: u8 = 0x04;
pub const MS_GENERAL: u8 = 0x01;

// MIDI jack types
pub const EMBEDDED: u8 = 0x01;
pub const EXTERNAL: u8 = 0x02;
