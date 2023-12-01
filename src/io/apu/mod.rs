use crate::cpu;

use self::channel::Channel;

use super::{map::apu_io::WAVE_SIZE, Bus};

pub mod channel;
pub mod duty;
pub mod envelope;
pub mod length;
pub mod noise;
pub mod sweep;
pub mod timer;
pub mod wave;

const AMP_MAX: i16 = i16::MAX / 16;
const AMP_CHL: i16 = AMP_MAX / 4;
const AMP_BASE: i16 = AMP_CHL / 16;

pub const CHANNELS: usize = 2;
pub const SAMPLE_RATE: usize = 48000;
pub const LATENCY: usize = 16;

pub const DUTY_TABLE: [[bool; 8]; 4] = [
    [false, false, false, false, false, false, false, true],
    [true, false, false, false, false, false, false, true],
    [true, false, false, false, false, true, true, true],
    [false, true, true, true, true, true, true, false],
];

#[derive(Default)]
pub struct Apu {
    left_volume: u8,
    right_volume: u8,

    // registers
    pub nr10: u8, // channel 1
    pub nr11: u8,
    pub nr12: u8,
    pub nr13: u8,
    pub nr14: u8,

    pub nr20: u8, // channel 2
    pub nr21: u8,
    pub nr22: u8,
    pub nr23: u8,
    pub nr24: u8,

    pub nr30: u8, // channel 3
    pub nr31: u8,
    pub nr32: u8,
    pub nr33: u8,
    pub nr34: u8,

    pub nr40: u8, // channel 4
    pub nr41: u8,
    pub nr42: u8,
    pub nr43: u8,
    pub nr44: u8,

    pub nr50: u8, // mixer
    pub nr51: u8,
    pub nr52: u8,

    ch1: Channel, // tone & sweep
    ch2: Channel, // tone
    ch3: Channel, // wave output
    ch4: Channel, // noise

    // wave memory
    pub wave: Box<[u8]>,

    // timing
    clock: u16,
    fs_clock: u16,

    // sequencing
    frame_sequence: u8,

    // mixing
    sample: usize,
    pub output_left: i16,
    pub output_right: i16,
    pub update: bool,

    enable: bool,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            wave: vec![0; WAVE_SIZE].into_boxed_slice(),
            ..Default::default()
        }
    }

    pub fn tick(&mut self) {
        self.update = true;

        if self.enable {
            self.clock += 1;
            self.fs_clock += 1;

            if self.ch1.duty.enable {
                self.ch1.duty.tick();
            }
            if self.ch2.duty.enable {
                self.ch2.duty.tick();
            }
            self.ch3.wave.tick(&self.wave);
            self.ch4.noise.tick();

            // output sample to buffer
            let cycles_per_sample = cpu::FREQUENCY / SAMPLE_RATE;
            if self.clock >= cycles_per_sample as u16 {
                self.output_left = 0;
                self.output_right = 0;

                self.ch1_sample();
                self.ch2_sample();
                self.ch3_sample();
                self.ch4_sample();

                self.sample += 1;
                self.update = true;
                self.clock -= cycles_per_sample as u16;
            }

            if self.fs_clock > 0x2000 {
                self.frame_sequencer();
                self.fs_clock -= 0x2000;
            }
        }
    }

    pub fn frame_sequencer(&mut self) {
        // length clock
        if self.frame_sequence & 1 == 0 {
            if self.ch1.enable {
                self.ch1.length_cycle();
            }
            if self.ch2.enable {
                self.ch2.length_cycle();
            }
            if self.ch3.enable {
                self.ch3.length_cycle();
            }
            if self.ch4.enable {
                self.ch4.length_cycle();
            }
        }

        // sweep clock
        if self.ch3.enable && (self.frame_sequence == 2 || self.frame_sequence == 6) {
            self.ch1
                .sweep
                .tick(&mut self.ch1.duty, &mut self.ch1.enable); // FIXME: check it should be ch1
        }

        // envelope clock
        if self.frame_sequence == 7 {
            self.ch1.envelope.tick();
            self.ch2.envelope.tick();
            self.ch4.envelope.tick();
        }

        self.frame_sequence = (self.frame_sequence + 1) % 8;
    }

    pub fn ch1_trigger(&mut self) {
        self.ch1.enable = true;
        self.ch1.duty.enable = true;
        if self.ch1.length.timer.counter == 0 {
            self.ch1.length.timer.counter = 64;
        }

        self.ch1.duty.timer.reset();
        self.ch1.envelope.timer.reset();
        self.ch1.sweep.timer.reset();

        self.ch1.envelope.enable = self.ch1.envelope.timer.period > 0;
        self.ch1.envelope.volume = self.ch1.envelope.start_volume;

        self.ch1.sweep.enable = self.ch1.sweep.shift > 0 || self.ch1.sweep.timer.period > 0;
        self.ch1.sweep.frequency = self.ch1.duty.frequency;
        self.ch1.sweep.calculated = false;

        if self.ch1.sweep.shift > 0 {
            let sweep_frequency = self.ch1.sweep.calc_frequency();
            if sweep_frequency >= 2048 {
                self.ch1.enable = false;
            }
        }

        if !self.ch1.dac {
            self.ch1.enable = false;
        }
    }

    pub fn ch2_trigger(&mut self) {
        self.ch2.enable = true;
        self.ch2.duty.enable = true;
        if self.ch2.length.timer.counter == 0 {
            self.ch2.length.timer.counter = 64;
        }

        self.ch2.duty.timer.reset();
        self.ch2.envelope.timer.reset();

        self.ch2.envelope.enable = self.ch2.envelope.timer.period > 0;
        self.ch2.envelope.volume = self.ch2.envelope.start_volume;

        if !self.ch2.dac {
            self.ch2.enable = false;
        }
    }

    pub fn ch3_trigger(&mut self) {
        self.ch3.enable = true;
        if self.ch3.length.timer.counter == 0 {
            self.ch3.length.timer.counter = 256;
        }

        self.ch3.wave.position = 0;

        self.ch3.wave.timer.reset();

        if !self.ch3.dac {
            self.ch3.enable = false;
        }
    }

    pub fn ch4_trigger(&mut self) {
        self.ch4.enable = true;

        self.ch4.noise.timer.reset();
        self.ch4.envelope.timer.reset();

        self.ch4.envelope.enable = self.ch4.envelope.timer.period > 0;
        self.ch4.envelope.volume = self.ch4.envelope.start_volume;

        self.ch4.noise.lfsr = 0xFFFF;

        if !self.ch4.dac {
            self.ch4.enable = false;
        }
    }

    pub fn ch1_sample(&mut self) {
        if self.ch1.enable {
            // println!("{}", self.ch1.duty.state);
            let output_left_temp = self.output_left as i32;
            let output_right_temp = self.output_right as i32;
            self.output_left = (output_left_temp - AMP_CHL as i32 / 2
                + AMP_BASE as i32
                    * self.ch1.duty.state as i32
                    * self.ch1.envelope.volume as i32
                    * self.ch1.left as i32
                    * self.left_volume as i32) as i16;
            self.output_right = (output_right_temp - AMP_CHL as i32 / 2
                + AMP_BASE as i32
                    * self.ch1.duty.state as i32
                    * self.ch1.envelope.volume as i32
                    * self.ch1.right as i32
                    * self.right_volume as i32) as i16;
            self.output_left = self.ch1.duty.state as i16 * 64;
        }
    }

    pub fn ch2_sample(&mut self) {
        if self.ch2.enable {
            let output_left_temp = self.output_left as i32;
            let output_right_temp = self.output_right as i32;
            self.output_left = (output_left_temp - AMP_CHL as i32 / 2
                + AMP_BASE as i32
                    * self.ch2.duty.state as i32
                    * self.ch2.envelope.volume as i32
                    * self.ch2.left as i32
                    * self.left_volume as i32) as i16;
            self.output_right = (output_right_temp - AMP_CHL as i32 / 2
                + AMP_BASE as i32
                    * self.ch2.duty.state as i32
                    * self.ch2.envelope.volume as i32
                    * self.ch2.right as i32
                    * self.right_volume as i32) as i16;
        }
    }

    pub fn ch3_sample(&mut self) {
        if self.ch3.enable {
            let output_left_temp = self.output_left as i32;
            let output_right_temp = self.output_right as i32;
            let output_adjusted = self.ch3.wave.output >> (self.ch3.wave.shift.wrapping_sub(1));
            self.output_left = (output_left_temp - AMP_CHL as i32 / 2
                + AMP_BASE as i32
                    * output_adjusted as i32
                    * self.ch3.left as i32
                    * self.left_volume as i32) as i16;
            self.output_right = (output_right_temp - AMP_CHL as i32 / 2
                + AMP_BASE as i32
                    * output_adjusted as i32
                    * self.ch3.right as i32
                    * self.right_volume as i32) as i16;
        }
    }

    pub fn ch4_sample(&mut self) {
        if self.ch4.enable {
            let output_left_temp = self.output_left as i32;
            let output_right_temp = self.output_right as i32;
            self.output_left = (output_left_temp - AMP_CHL as i32 / 2
                + AMP_BASE as i32
                    * self.ch4.noise.state as i32
                    * self.ch4.envelope.volume as i32
                    * self.ch4.left as i32
                    * self.left_volume as i32) as i16;
            self.output_right = (output_right_temp - AMP_CHL as i32 / 2
                + AMP_BASE as i32
                    * self.ch4.noise.state as i32
                    * self.ch4.envelope.volume as i32
                    * self.ch4.right as i32
                    * self.right_volume as i32) as i16;
        }
    }

    pub fn set_nr10(&mut self, value: u8) {
        self.nr10 = value;
        self.ch1.sweep.timer.period = (value as u16 & 0x70) >> 4;
        if value & (1 << 3) != 0 && self.ch1.sweep.decreasing && self.ch1.sweep.calculated {
            self.ch1.enable = false;
        }

        self.ch1.sweep.decreasing = value & (1 << 3) != 0;
        self.ch1.sweep.shift = value & 7;
    }

    pub fn set_nr11(&mut self, value: u8) {
        self.nr11 = value;

        self.ch1.duty.pattern = (value & 0xc0) >> 6;
        self.ch1.length.timer.period = 64 - (value as u16 & 0x3f);
    }

    pub fn set_nr12(&mut self, value: u8) {
        self.nr12 = value;

        self.ch1.dac = value & 0xf8 != 0;
        self.ch1.envelope.start_volume = (value & 0xf0) >> 4;
        self.ch1.envelope.direction = if value & 8 != 0 { 1 } else { -1 };
        self.ch1.envelope.timer.period = value as u16 & 7;

        if !self.ch1.dac {
            self.ch1.enable = false;
        }
    }

    pub fn set_nr13(&mut self, value: u8) {
        self.nr13 = value;

        self.ch1.duty.frequency &= 0xff00;
        self.ch1.duty.frequency |= value as u16;
    }

    pub fn set_nr14(&mut self, value: u8) {
        self.nr14 = value;

        self.ch1.length.enable = value & 0x40 != 0;
        self.ch1.duty.frequency &= 0x00ff;
        self.ch1.duty.frequency |= (value as u16 & 7) << 8;

        if value & 0x80 != 0 {
            self.ch1_trigger();
        }
    }

    pub fn set_nr50(&mut self, value: u8) {
        self.nr50 = value;
        self.right_volume = value & 0x07;
        self.left_volume = (value & 0x70) >> 4;
    }

    pub fn set_nr51(&mut self, value: u8) {
        self.nr51 = value;
        self.ch1.right = (value & (1 << 0)) >> 0 != 0;
        self.ch2.right = (value & (1 << 1)) >> 1 != 0;
        self.ch3.right = (value & (1 << 2)) >> 2 != 0;
        self.ch4.right = (value & (1 << 3)) >> 3 != 0;
        self.ch1.right = (value & (1 << 4)) >> 4 != 0;
        self.ch2.right = (value & (1 << 5)) >> 5 != 0;
        self.ch3.right = (value & (1 << 6)) >> 6 != 0;
        self.ch4.right = (value & (1 << 7)) >> 7 != 0;
    }

    pub fn set_nr52(&mut self, value: u8) {
        self.nr52 = value;

        self.enable = value & 0x80 != 0;

        if !self.enable {
            *self = Self::new();
        }
    }
}
