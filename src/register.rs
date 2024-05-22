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
        self.af = (self.af & 0x00fF) | (value as u16) << 8;
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
        self.bc = (self.bc & 0x00ff) | value as u16;
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
        self.de = (self.de & 0x00ff) | value as u16;
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
        self.hl = (self.hl & 0x00ff) | value as u16;
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
