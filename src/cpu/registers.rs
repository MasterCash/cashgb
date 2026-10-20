use crate::cpu::instructions::Interrupt;

#[derive(Clone, Copy)]
pub struct Register {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
}

impl Register {
    pub fn new() -> Self {
        Self {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
        }
    }
    pub fn get_a(&self) -> u8 {
        (self.af >> 8) as u8
    }

    pub fn set_a(&mut self, value: u8) {
        self.af = (self.af & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_f(&self) -> u8 {
        self.af as u8
    }

    pub fn set_f(&mut self, value: u8) {
        self.af = (self.af & 0xff00) | value as u16;
    }

    pub fn get_af(&self) -> u16 {
        self.af
    }

    pub fn set_af(&mut self, value: u16) {
        self.af = value;
    }

    pub fn get_b(&self) -> u8 {
        (self.bc >> 8) as u8
    }

    pub fn set_b(&mut self, value: u8) {
        self.bc = (self.bc & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_c(&self) -> u8 {
        self.bc as u8
    }

    pub fn set_c(&mut self, value: u8) {
        self.bc = (self.bc & 0xff00) | value as u16;
    }

    pub fn get_bc(&self) -> u16 {
        self.bc
    }

    pub fn set_bc(&mut self, value: u16) {
        self.bc = value;
    }

    pub fn get_d(&self) -> u8 {
        (self.de >> 8) as u8
    }

    pub fn set_d(&mut self, value: u8) {
        self.de = (self.de & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_e(&self) -> u8 {
        self.de as u8
    }

    pub fn set_e(&mut self, value: u8) {
        self.de = (self.de & 0xff00) | value as u16;
    }

    pub fn get_de(&self) -> u16 {
        self.de
    }

    pub fn set_de(&mut self, value: u16) {
        self.de = value;
    }

    pub fn get_h(&self) -> u8 {
        (self.hl >> 8) as u8
    }

    pub fn set_h(&mut self, value: u8) {
        self.hl = (self.hl & 0x00FF) | ((value as u16) << 8);
    }

    pub fn get_l(&self) -> u8 {
        self.hl as u8
    }

    pub fn set_l(&mut self, value: u8) {
        self.hl = (self.hl & 0xff00) | value as u16;
    }

    pub fn get_hl(&self) -> u16 {
        self.hl
    }

    pub fn set_hl(&mut self, value: u16) {
        self.hl = value;
    }
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

// Timer Mode from TAC register indicates the frequency of the timer. The values are in M-Cycles (machine cycles).
pub enum TimerMode {
    Mode0 = 256, // 4096 Hz
    Mode1 = 4,   // 262144 Hz
    Mode2 = 16,  // 65536 Hz
    Mode3 = 64,  // 16384 Hz
}

pub struct TimerRegister {
    // count of M-Cycles since last DIV increment
    pub(crate) div_cycle_count: u8,
    // Timer Divider Register (DIV) - increments at 16384 Hz or once every 64 M-Cycles
    pub(crate) div: u8,
    // count of M-Cycles since last TIMA increment
    pub(crate) tima_cycle_count: u8,
    // Timer Counter (TIMA) - increments at a rate determined by TAC
    pub(crate) tima: u8,
    // Timer Modulo (TMA) - value to which TIMA is reset when it overflows
    pub(crate) tma: u8,
    // Timer Control (TAC) - controls the timer's operation
    pub(crate) tac: u8,
}

impl TimerRegister {
    pub fn new() -> Self {
        Self {
            div_cycle_count: 0,
            tima_cycle_count: 0,
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub fn step(&mut self) -> Option<Interrupt> {
        self.div_cycle_count = self.div_cycle_count.wrapping_add(1);
        if self.div_cycle_count >= 64 {
            self.div_cycle_count = 0;
            self.div = self.div.wrapping_add(1);
        }

        if !self.is_timer_enabled() {
            return None;
        }

        self.tima_cycle_count = self.tima_cycle_count.wrapping_add(1);
        let timer_mode = self.get_timer_mode();
        if self.tima_cycle_count >= timer_mode as u8 {
            self.tima_cycle_count = 0;
            if self.tima == 0xFF {
                self.tima = self.tma;
                return Some(Interrupt::Timer);
            } else {
                self.tima = self.tima.wrapping_add(1);
            }
        }
        None
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xff04 => self.div = 0, // Writing to DIV resets it
            0xff05 => self.tima = value,
            0xff06 => self.tma = value,
            0xff07 => self.tac = value & 0x07, // Only lower 3 bits are used
            _ => {}
        }
    }

    pub fn read(&self, addr: u16) -> Option<u8> {
        match addr {
            0xff04 => Some(self.div),
            0xff05 => Some(self.tima),
            0xff06 => Some(self.tma),
            0xff07 => Some(self.tac | 0xF8), // Upper 5 bits are always read as 1
            _ => None,
        }
    }

    pub fn is_timer_enabled(&self) -> bool {
        (self.tac & 0x04) != 0
    }

    pub fn get_timer_mode(&self) -> TimerMode {
        match self.tac & 0x03 {
            0 => TimerMode::Mode0,
            1 => TimerMode::Mode1,
            2 => TimerMode::Mode2,
            3 => TimerMode::Mode3,
            _ => unreachable!(),
        }
    }
}

impl Default for TimerRegister {
    fn default() -> Self {
        Self::new()
    }
}
