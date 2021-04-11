use crate::cartridge::Cartridge;
use crate::RefCell;
use crate::Rc;

enum CounterMode {
    Zero,
    One,
}
pub enum Frame {
    None,
    Quarter,
    Half,
}

pub const WAVEFORMS: [[u8; 8]; 4] = [[0, 1, 0, 0, 0, 0, 0, 0],[0, 1, 1, 0, 0, 0, 0, 0],[0, 1, 1, 1, 1, 0, 0, 0],[1, 0, 0, 1, 1, 1, 1, 1]];

pub const PERIODS: [u8; 16] = [214, 190, 170, 160, 143, 127, 113, 107, 95, 80, 71, 64, 53, 42, 36, 27];

//https://wiki.nesdev.com/w/index.php/APU_Length_Counter#:
pub const LENGTHS: [u8; 32] =[  
    10, 254, 20, 2, 40, 4, 80, 6,
    160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30];

//https://wiki.nesdev.com/w/index.php/APU_Triangle
pub const TRIANGLE_SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8,
    7, 6, 5, 4, 3, 2, 1, 0,
    0, 1, 2, 3, 4, 5, 6, 7,
    8, 9, 10, 11, 12, 13, 14, 15];

//https://wiki.nesdev.com/w/index.php/APU_Noise
const NOISE_TIMER: [u16; 16] = [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

                    
const PI: f64 = 3.141592;

bitfield!{
    pub struct EnvelopeRegister(u8);
    pub constant_volume,   _: 3, 0;
    pub decay,     _: 3, 0;
    pub reload,         _:    4;
    pub loop_flag,          _:    5;
}

pub struct APU {
    pub samples: Vec<i16>,
    pulse_0: PULSE,
    pulse_1: PULSE,
    triangle: TRIANGLE,
    noise: NOISE,
    global_time: u32,

    pub counter: i64,
    pub cycles: u64,
    pub irq: bool,
    pub public_irq: bool,
    pub private_irq: bool,
    counter_mode: CounterMode,
}

impl APU {
    pub fn new() -> APU {
        APU {
            pulse_0: PULSE::new(false), 
            pulse_1: PULSE::new(true),
            triangle: TRIANGLE::new(),
            noise: NOISE::new(),
            global_time: 0,
            samples: Vec::new(),

            counter: 0,
            cycles: 0,
            irq: true,
            public_irq: false,
            private_irq: false,
            counter_mode: CounterMode::Zero,
        }
    }
    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x4000 => {
                self.pulse_0.duty_cycle = data as usize >> 6;
                self.pulse_0.envelope.controller = EnvelopeRegister(data);
                self.pulse_0.length_counter.pending_stop = Some(data & 0x20 != 0);
            }
            0x4001 => {
                self.pulse_0.sweeper.enabled = data & 0x80 != 0;
                self.pulse_0.sweeper.period = (data & 0x70) >> 4;
                self.pulse_0.sweeper.negate = data & 0x8 != 0;
                self.pulse_0.sweeper.shift = data & 0x7;
                self.pulse_0.sweeper.reload = true;
            }
            0x4002 => {
                self.pulse_0.sequencer.decay = (self.pulse_0.sequencer.decay & 0xFF00) | data as u16;
            }
            0x4003 =>
             {
                self.pulse_0.length_counter.pending_register = Some(data);
                self.pulse_0.sequencer.decay = (self.pulse_0.sequencer.decay & 0x00FF) | ((data as u16 & 0x7) << 8);
                self.pulse_0.envelope.restart = true;
                self.pulse_0.sequencer.current_step = 0; 
            }
            0x4004 => {
                self.pulse_1.duty_cycle = data as usize >> 6;
                self.pulse_1.envelope.controller = EnvelopeRegister(data);
                self.pulse_1.length_counter.pending_stop = Some(data & 0x20 != 0);
            }
            0x4005 => {
                self.pulse_1.sweeper.enabled = data & 0x80 != 0;
                self.pulse_1.sweeper.period = (data & 0x70) >> 4;
                self.pulse_1.sweeper.negate = data & 0x8 != 0;
                self.pulse_1.sweeper.shift = data & 0x7;
                self.pulse_1.sweeper.reload = true;
            }
            0x4006 => {
                self.pulse_1.sequencer.decay = (self.pulse_1.sequencer.decay & 0xFF00) | data as u16;
            }
            0x4007 =>
             {
                self.pulse_1.length_counter.pending_register = Some(data);
                self.pulse_1.sequencer.decay = (self.pulse_1.sequencer.decay & 0x00FF) | ((data as u16 & 0x7) << 8);
                self.pulse_1.envelope.restart = true;
                self.pulse_1.sequencer.current_step = 0; 
            }
            0x4008 => 
            {
                self.triangle.control_flag = data & 0x80 != 0;
                self.triangle.length_counter.pending_stop = Some(data & 0x20 != 0);
                self.triangle.linear_counter_period = data & 0x7F;
            }
            0x4009 => 
            {

            }
            0x4010 => {

            }
            0x4011 => {
            }
            0x4012 => {
            }
            0x4013 => {
            }  
            0x400A => 
            {

                self.triangle.sequencer.decay = (self.triangle.sequencer.decay & 0xFF00) | data as u16;
            }
            0x400B => 
            {
                self.triangle.length_counter.pending_register = Some(data);
                self.triangle.sequencer.decay = (self.triangle.sequencer.decay & 0x00FF) | ((data as u16 & 0x7) << 8);
                self.triangle.linear_counter_start = true;
            }
            0x400C => 
            {
                self.noise.length_counter.pending_stop = Some(data & 0x20 != 0);
                self.noise.envelope.controller = EnvelopeRegister(data);
            },
            0x400D => {},
            0x400E => 
            {
                self.noise.mode = data & 0x80 != 0;
                self.noise.period = NOISE_TIMER[data as usize & 0xF];
            },
            0x400F => 
            {
                self.noise.length_counter.pending_register = Some(data);
                self.noise.envelope.restart = true;
            },
            0x4015 => {
                self.pulse_0.length_counter.enable(data & 0x1 != 0);
                self.pulse_1.length_counter.enable(data & 0x2 != 0);
                self.triangle.length_counter.enable(data & 0x3 != 0);
                self.noise.length_counter.enable(data & 0x4 != 0);
            }
            0x4017 => {
                self.irq = data & 0x40 == 0;
                if !self.irq
                {
                    self.public_irq = false;
                    self.private_irq = false;
                }
        
                self.counter_mode = if data & 0x80 == 0 {
                    CounterMode::Zero
                } else {
                    CounterMode::One
                };
        
                self.counter = if self.cycles & 1 == 0 { 0 } else { -1 };
        
                let frame = match self.counter_mode {
                    CounterMode::Zero => Frame::None,
                    CounterMode::One => Frame::Half,
                };
                self.match_frame(frame);
            }
            _ => {},
        }
    }
    pub fn clock(&mut self) 
    {
            self.triangle.clock_sequencer();
            if self.global_time % 2 == 1 
            {
                self.pulse_0.sequencer.clock(true);
                self.pulse_1.sequencer.clock(true);
                self.noise.clock();
            }
            let frame = match self.counter_mode {
                CounterMode::Zero => self.clock_zero(),
                CounterMode::One => self.clock_one(),
            };
            self.counter += 1;
            self.match_frame(frame);

            self.pulse_0.length_counter.update();();
            self.pulse_1.length_counter.update();();
            self.triangle.length_counter.update();
            self.noise.length_counter.update();
    
            if self.global_time % 40 == 0 {
                let sample = self.sample();
                self.samples.push(sample);
                self.samples.push(sample);
            }
        self.global_time += 1;
    }

    fn clock_zero(&mut self) -> Frame {
        match self.counter {
            7460 => Frame::Quarter,
            14915 => Frame::Half,
            22374 => Frame::Quarter,
            29831 => {
                if self.irq 
                {
                    self.private_irq = true;
                }
                Frame::None
            }
            29832 => {
                if self.irq 
                {
                    self.private_irq = true;
                    self.public_irq = true;
                }else
                {
                    self.public_irq = false;
                }
                Frame::Half
            }
            29833 => {
                if self.irq 
                {
                    self.private_irq = true;
                    self.public_irq = true;
                }else
                {
                    self.public_irq = false;
                }
                self.counter = 2;
                Frame::None
            }
            _ => Frame::None,
        }
    }
    fn clock_one(&mut self) -> Frame {
        match self.counter {
            74560 => Frame::Quarter,
            14916 => Frame::Half,
            22374 => Frame::Quarter,
            37284 => {
                self.counter = 1;
                Frame::Half
            }
            _ => Frame::None,
        }
    }

    fn match_frame(&mut self, result: Frame) {
        match result {
            Frame::Quarter => 
            {
                self.pulse_0.envelope.clock();
                self.pulse_1.envelope.clock();();
                self.triangle.clock_quarter();
            }
            Frame::Half => 
            {
                self.pulse_0.envelope.clock();
                self.pulse_0.length_counter.clock();
                self.pulse_0.sweeper.clock(&mut self.pulse_0.sequencer);

                self.pulse_1.envelope.clock();
                self.pulse_1.length_counter.clock();
                self.pulse_1.sweeper.clock(&mut self.pulse_0.sequencer);

                self.triangle.clock_quarter();
                self.triangle.length_counter.clock();

                self.noise.clock_quarter();
                self.noise.clock_half();
            }
            Frame::None => (),
        }
    }

    fn sample(&mut self) -> i16 {
        let pulse_0 = self.pulse_0.sample() as f64;
        let pulse_1 = self.pulse_1.sample() as f64;
        let triangle = self.triangle.sample() as f64;
        let noise = self.noise.sample() as f64;

        let pulse_out = 95.88 / ((8218.0 / (pulse_0 + pulse_1)) + 100.0);
        let tnd_out = 159.79 / ((1.0 / (triangle / 8227.0 + noise / 12241.0 + 0.0 / 22638.0)) + 100.0);
       //let tnd_out = 159.79 / ((1.0 / (0.0 / 8227.0 + 0.0 / 12241.0 + dmc / 22638.0)) + 100.0);
        let mut output = (pulse_out + tnd_out) * 65535.0;



        return output as i16;
    }
}

pub struct PULSE {
    sweeper: SWEEPER,
    envelope: ENVELOPE,
    sequencer: SEQUENCER,
    length_counter: LENGTH_COUNTER,
    duty_cycle: usize,
}

impl PULSE {
    pub fn new(negation: bool) -> PULSE {
        PULSE {
            sweeper: SWEEPER::new(negation),
            envelope: ENVELOPE::new(),
            sequencer: SEQUENCER::new(8),
            length_counter: LENGTH_COUNTER::new(),
            duty_cycle: 0,
        }
    }

    #[allow(unused_assignments)]
    pub fn sample(&self) -> u8 {
        let mut period = 0;
        if self.sweeper.negate == false
        {
            period = self.sequencer.decay + (self.sequencer.decay >> self.sweeper.shift)
        } else 
        {
            period = self.sequencer.decay - (self.sequencer.decay >> self.sweeper.shift) - self.sweeper.negation_mode as u16
        }

        if (self.length_counter.enabled && self.length_counter.counter > 0) && self.sequencer.decay >= 8 && period < 0x800
        {
            return WAVEFORMS[self.duty_cycle][self.sequencer.current_step] * self.envelope.volume();
        } else {
            return 0;
        }
    }
}


pub struct SEQUENCER {
    pub reload: u16,
    pub decay: u16,
    steps: usize,
    pub current_step: usize,
}

impl SEQUENCER {
    pub fn new(output_length: usize) -> SEQUENCER {
        SEQUENCER {
            reload: 0,
            decay: 0,
            current_step: 0,
            steps: output_length,
        }
    }

    pub fn clock(&mut self, step_enabled: bool) -> bool {
        if self.reload == 0 {
            self.reload = self.decay;
            if step_enabled {
                self.current_step = (self.current_step + 1) % self.steps;
            }
            true
        } else {
            self.reload -= 1;
            false
        }
    }
}

pub struct SWEEPER {
    enabled: bool,
    reload: bool,
    shift: u8,
    negate: bool,
    negation_mode: bool,
    period: u8,
    counter: u8,
}

impl SWEEPER {
    pub fn new(negation_mode: bool) -> SWEEPER {
        SWEEPER {
            enabled: false,
            reload: false,
            shift: 0,
            negate: false,
            period: 0,
            counter: 0,
            negation_mode,
        }
    }

    #[allow(unused_assignments)]
    pub fn clock(&mut self, sequencer: &mut SEQUENCER) {
        if self.counter == 0 && self.enabled && self.shift > 0 && sequencer.decay >= 8 
        {
            let mut period = 0;
            if self.negate == false
            {
                period = sequencer.decay + (sequencer.decay >> self.shift)
            } else 
            {
                period = sequencer.decay - (sequencer.decay >> self.shift) - self.negation_mode as u16
            }
            if period < 0x800 {
                sequencer.decay = period;
                sequencer.decay = period;
            }
        }
        if self.counter == 0 || self.reload 
        {
            self.counter = self.period;
            self.reload = false;
        } else 
        {
            self.counter -= 1;
        }
    }

}

pub struct LENGTH_COUNTER {
    counter: u8,
    pub enabled: bool,
    stopped: bool,
    pending_stop: Option<bool>,
    pending_register: Option<u8>,
}

impl LENGTH_COUNTER {
    pub fn new() -> LENGTH_COUNTER {
        LENGTH_COUNTER {
            counter: 0,
            enabled: false,
            stopped: false,
            pending_stop: None,
            pending_register: None,
        }
    }


    pub fn enable(&mut self, v: bool) {
        self.enabled = v;
        if !v {
            self.counter = 0;
        }
    }

    pub fn update(&mut self) {
        if let Some(v) = self.pending_stop {
            self.stopped = v;
            self.pending_stop = None;
        }

        if let Some(value) = self.pending_register {
            if self.enabled {
                self.counter = LENGTHS[(value >> 3) as usize];
            }
            self.pending_register = None;
        }
    }

    pub fn clock(&mut self) {
        if let Some(_) = self.pending_register {
            if self.counter == 0 {
                return;
            } else {
                self.pending_register = None;
            }
        }
        if self.enabled && !self.stopped && self.counter > 0 {
            self.counter -= 1;
        }
    }

}

pub struct ENVELOPE {
    controller: EnvelopeRegister,
    counter: u8,
    level: u8,
    restart: bool,
}

impl ENVELOPE {
    pub fn new() -> ENVELOPE {
        ENVELOPE {
            counter: 0,
            level: 0,
            controller: EnvelopeRegister(0),
            restart: false,
        }
    }

    pub fn clock(&mut self) {
        if self.restart 
        {
            self.restart = false;
            self.level = 0x0F;
            self.counter = self.controller.decay();
        } else 
        {
            if self.counter > 0 {
                self.counter -= 1;
            } else 
            {
                if self.level > 0 {
                    let level = self.level - 1;
                    self.level = level & 0x0F;
                    self.counter = self.controller.decay();
                } else if self.controller.loop_flag() 
                {
                    self.level = 0x0F;
                    self.counter = self.controller.decay();
                }
            }
        }
    }

    pub fn volume(&self) -> u8 {
        if self.controller.reload() {
            return self.controller.constant_volume();
        } else {
            return self.level;
        }
    }
}

pub struct TRIANGLE {
    length_counter: LENGTH_COUNTER,
    sequencer: SEQUENCER,
    linear_counter: u8,
    linear_counter_start: bool,
    linear_counter_period: u8,
    control_flag: bool,
}

impl TRIANGLE {
    pub fn new() -> TRIANGLE {
        TRIANGLE {
            length_counter: LENGTH_COUNTER::new(),
            sequencer: SEQUENCER::new(32),
            linear_counter: 0,
            control_flag: false,
            linear_counter_period: 0,
            linear_counter_start: false,
        }
    }

    pub fn sample(&self) -> u8 {
        let active = (self.length_counter.enabled && self.length_counter.counter > 0) && self.linear_counter > 0;
        if active && self.sequencer.decay > 2 
        {
            return TRIANGLE_SEQUENCE[self.sequencer.current_step];
        } else {
            return 0;
        }
    }

    pub fn clock_sequencer(&mut self) {
        let active = (self.length_counter.enabled && self.length_counter.counter > 0) && self.linear_counter > 0;
        self.sequencer.clock(active);
    }

    pub fn clock_quarter(&mut self) {
        if self.linear_counter_start {
            self.linear_counter = self.linear_counter_period;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.control_flag {
            self.linear_counter_start = false;
        }
    }
}

pub struct NOISE {
    envelope: ENVELOPE,
    length_counter: LENGTH_COUNTER,
    mode: bool,
    period: u16,
    counter: u16,
    shift: u16,
}

impl NOISE {
    pub fn new() -> NOISE {
        NOISE {
            envelope: ENVELOPE::new(),
            length_counter: LENGTH_COUNTER::new(),
            mode: false,
            period: 0,
            counter: 0,
            shift: 1,
        }
    }


    pub fn sample(&self) -> u8 {
        if (self.length_counter.enabled && self.length_counter.counter > 0) && self.shift & 1 == 0 {
            return self.envelope.volume();
        } else {
            return 0;
        }
    }

    pub fn clock(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
        } else {
            self.counter = self.period;
            let bit1 = (self.shift >> (if self.mode { 6 } else { 1 })) & 1;
            let bit2 = self.shift & 1;
            self.shift = (self.shift >> 1) | (bit1 ^ bit2) << 14
        }
    }

    pub fn clock_quarter(&mut self) {
        self.envelope.clock();
    }

    pub fn clock_half(&mut self) {
        self.length_counter.clock();
    }
}





