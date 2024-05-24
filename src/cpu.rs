use std::fmt::Display;

use crate::cart::Cart;
use crate::register::Register;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadTarget {
    A,
    BC,
    DE,
    HL,
    SP,
    BCAddr,
    DEAddr,
    HLAddrInc,
    HLAddrDec,
    HLAddr,
    B,
    D,
    H,
    PCAddr,
    PC16Addr,
    C,
    E,
    L,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadSource {
    PCAddr,
    HL,
    SPE,
    BCAddr,
    DEAddr,
    HLAddrInc,
    HLAddrDec,
    SP,
    A,
    PC,
    PC16,
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementTarget {
    BC,
    DE,
    HL,
    SP,
    B,
    D,
    H,
    HLAddr,
    C,
    E,
    L,
    A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecrementTarget {
    B,
    D,
    H,
    HLAddr,
    BC,
    DE,
    HL,
    SP,
    C,
    E,
    L,
    A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpCondition {
    None,
    Z,
    C,
    NZ,
    NC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddTarget {
    SP,
    A,
    HL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddSource {
    BC,
    DE,
    HL,
    SP,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    PC,
    PCSigned,
    HLAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddCarrySource {
    PC,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtractCarrySource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
    PC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtractSource {
    A,
    B,
    C,
    D,
    E,
    H,
    PC,
    L,
    HLAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndSource {
    PC,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
    PC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XOrSource {
    PC,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrSource {
    PC,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopTarget {
    BC,
    DE,
    HL,
    AF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushTarget {
    BC,
    DE,
    HL,
    AF,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadAccumulatorTarget {
    PCAddr,
    A,
    CAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadAccumulatorSource {
    A,
    PCAddr,
    CAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitwiseSource {
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
    A,
}

#[derive(Debug)]
pub enum Instruction {
    Nop,
    Stop,
    Halt,
    DisableInterrupts,
    EnableInterrupts,
    Add(AddTarget, AddSource),
    AddCarry(AddCarrySource),
    Subtract(SubtractSource),
    SubtractCarry(SubtractCarrySource),
    And(AndSource),
    Or(OrSource),
    XOr(XOrSource),
    Load(LoadTarget, LoadSource),
    LoadAccumulator(LoadAccumulatorTarget, LoadAccumulatorSource),
    Increment(IncrementTarget),
    Decrement(DecrementTarget),
    RotateLeftCircular(BitwiseSource),
    RotateLeft(BitwiseSource),
    DecimalAdjustAccumulator,
    SetCarryFlag,
    JumpRelative(JumpCondition),
    Compare(CompareSource),
    RotateRightCircular(BitwiseSource),
    RotateRight(BitwiseSource),
    ShiftRightArithmetic(BitwiseSource),
    ShiftRightLogical(BitwiseSource),
    ShiftLeftArithmetic(BitwiseSource),
    Swap(BitwiseSource),
    Bit(u8, BitwiseSource),
    ComplementAccumulator,
    ComplementCarryFlag,
    Return(JumpCondition),
    ReturnInterrupt,
    Jump(JumpCondition),
    JumpHL,
    Pop(PopTarget),
    Push(PushTarget),
    Call(JumpCondition),
    Restart(u16),
    CB,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Nop => write!(f, "NOP"),
            Instruction::Stop => write!(f, "STOP"),
            Instruction::Halt => write!(f, "HALT"),
            Instruction::DisableInterrupts => write!(f, "DI"),
            Instruction::EnableInterrupts => write!(f, "EI"),
            Instruction::Add(t, s) => write!(f, "ADD {:?}, {:?}", t, s),
            Instruction::AddCarry(s) => write!(f, "ADC {:?}", s),
            Instruction::Subtract(s) => write!(f, "SUB A, {:?}", s),
            Instruction::SubtractCarry(s) => write!(f, "SBC A , {:?}", s),
            Instruction::And(s) => write!(f, "AND {:?}", s),
            Instruction::Or(s) => write!(f, "OR {:?}", s),
            Instruction::XOr(s) => write!(f, "XOR {:?}", s),
            Instruction::Load(t, s) => write!(f, "LD {:?}, {:?}", t, s),
            Instruction::LoadAccumulator(t, s) => write!(f, "LDH {:?}, {:?}", t, s),
            Instruction::Increment(s) => write!(f, "INC {:?}", s),
            Instruction::Decrement(s) => write!(f, "DEC {:?}", s),
            Instruction::RotateLeftCircular(s) => write!(f, "RLC {:?}", s),
            Instruction::RotateLeft(s) => write!(f, "RL {:?}", s),
            Instruction::DecimalAdjustAccumulator => write!(f, "DAA"),
            Instruction::SetCarryFlag => write!(f, "SCF"),
            Instruction::JumpRelative(c) => write!(f, "JR {:?}, e", c),
            Instruction::Compare(s) => write!(f, "CP {:?}", s),
            Instruction::RotateRightCircular(s) => write!(f, "RRC {:?}", s),
            Instruction::RotateRight(s) => write!(f, "RR {:?}", s),
            Instruction::ComplementAccumulator => write!(f, "CPL"),
            Instruction::ComplementCarryFlag => write!(f, "CCF"),
            Instruction::Return(c) => write!(f, "RET {:?}", c),
            Instruction::ReturnInterrupt => write!(f, "RETI"),
            Instruction::Jump(c) => write!(f, "JP {:?} nn", c),
            Instruction::JumpHL => write!(f, "JP HL"),
            Instruction::Pop(t) => write!(f, "POP {:?}", t),
            Instruction::Push(t) => write!(f, "PUSH {:?}", t),
            Instruction::Call(c) => write!(f, "CALL {:?}", c),
            Instruction::Restart(c) => write!(f, "RST {:#x}", c),
            Instruction::CB => write!(f, "CB"),
            Instruction::ShiftRightArithmetic(s) => write!(f, "SRA {:?}", s),
            Instruction::ShiftRightLogical(s) => write!(f, "SRL {:?}", s),
            Instruction::ShiftLeftArithmetic(s) => write!(f, "SLA {:?}", s),
            Instruction::Swap(s) => write!(f, "SWAP {:?}", s),
            Instruction::Bit(b, s) => write!(f, "BIT {:#x} {:?}", b, s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuStatus {
    Running,
    Stopped,
    Errored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Z = 1 << 7,
    N = 1 << 6,
    H = 1 << 5,
    C = 1 << 4,
}

pub struct Cpu {
    status: CpuStatus,
    register: Register,
    program_counter: u16,
    stack_pointer: u16,
    cart: Cart,
    boot_off: bool,
    boot_rom: [u8; 0x100],
    step_count: u8,
    instruction: Instruction,
    v_ram: [[u8; 0x1FFF]; 2],
    v_ram_bank: u8,
    w_ram: [[u8; 0x1FFF]; 8],
    w_ram_bank: u8,
    ime: bool,
}

impl Cpu {
    pub fn step(&mut self) {
        if CpuStatus::Errored == self.status || self.status == CpuStatus::Stopped {
            return;
        }
        if self.step_count == 0 {
            (self.instruction, self.step_count) =
                Cpu::get_instruction(&self.read(&self.program_counter));
            self.program_counter += 1;
        }
        if self.step_count > 1 {
            self.step_count -= 1;
            return;
        }

        self.process_instruction();
    }

    fn process_instruction(&mut self) {
        println!("Executing Instruction: {}", self.instruction);
        self.step_count = 0;
        match self.instruction {
            Instruction::Compare(source) => self.compare(source),
            Instruction::XOr(source) => self.xor(source),
            Instruction::Load(target, source) => self.load(target, source),
            Instruction::CB => {
                let (instruction, cycles) =
                    Cpu::get_cb_instruction(&self.read(&self.program_counter));
                self.instruction = instruction;
                self.step_count = cycles;
                self.program_counter += 1;
            }
            Instruction::Bit(bit, source) => self.bit(bit, source),
            Instruction::JumpRelative(condition) => self.jump_relative(condition),
            Instruction::EnableInterrupts => self.ime = true,
            Instruction::DisableInterrupts => self.ime = false,
            Instruction::LoadAccumulator(target, source) => self.load_accumulator(target, source),
            Instruction::Increment(target) => self.increment(target),
            Instruction::Call(condition) => self.call(condition),
            Instruction::Subtract(source) => self.sub(source),
            Instruction::Nop => (),
            Instruction::Push(target) => self.push(target),
            Instruction::RotateLeft(source) => self.rotate_Left(source),
            _ => panic!("unimplemented instruction: {}", self.instruction),
        }
    }

    pub fn new(cart: Cart) -> Self {
        Self {
            status: CpuStatus::Running,
            step_count: 0,
            ime: false,
            instruction: Instruction::Nop,
            cart,
            program_counter: 0,
            stack_pointer: 0,
            register: Register::new(),
            boot_off: false,
            v_ram: [[0; 0x1fff]; 2],
            v_ram_bank: 0,
            w_ram: [[0; 0x1fff]; 8],
            w_ram_bank: 1,
            boot_rom: [
                0x31, 0xfe, 0xff, 0xaf, 0x21, 0xff, 0x9f, 0x32, 0xcb, 0x7c, 0x20, 0xfb, 0x21, 0x26,
                0xff, 0x0e, 0x11, 0x3e, 0x80, 0x32, 0xe2, 0x0c, 0x3e, 0xf3, 0xe2, 0x32, 0x3e, 0x77,
                0x77, 0x3e, 0xfc, 0xe0, 0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1a, 0xcd, 0x95,
                0x00, 0xcd, 0x96, 0x00, 0x13, 0x7b, 0xfe, 0x34, 0x20, 0xf3, 0x11, 0xd8, 0x00, 0x06,
                0x08, 0x1a, 0x13, 0x22, 0x23, 0x05, 0x20, 0xf9, 0x3e, 0x19, 0xea, 0x10, 0x99, 0x21,
                0x2f, 0x99, 0x0e, 0x0c, 0x3d, 0x28, 0x08, 0x32, 0x0d, 0x20, 0xf9, 0x2e, 0x0f, 0x18,
                0xf3, 0x67, 0x3e, 0x64, 0x57, 0xe0, 0x42, 0x3e, 0x91, 0xe0, 0x40, 0x04, 0x1e, 0x02,
                0x0e, 0x0c, 0xf0, 0x44, 0xfe, 0x90, 0x20, 0xfa, 0x0d, 0x20, 0xf7, 0x1d, 0x20, 0xf2,
                0x0e, 0x13, 0x24, 0x7c, 0x1e, 0x83, 0xfe, 0x62, 0x28, 0x06, 0x1e, 0xc1, 0xfe, 0x64,
                0x20, 0x06, 0x7b, 0xe2, 0x0c, 0x3e, 0x87, 0xe2, 0xf0, 0x42, 0x90, 0xe0, 0x42, 0x15,
                0x20, 0xd2, 0x05, 0x20, 0x4f, 0x16, 0x20, 0x18, 0xcb, 0x4f, 0x06, 0x04, 0xc5, 0xcb,
                0x11, 0x17, 0xc1, 0xcb, 0x11, 0x17, 0x05, 0x20, 0xf5, 0x22, 0x23, 0x22, 0x23, 0xc9,
                0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c,
                0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6,
                0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc,
                0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e, 0x3c, 0x42, 0xb9, 0xa5, 0xb9, 0xa5, 0x42, 0x3c,
                0x21, 0x04, 0x01, 0x11, 0xa8, 0x00, 0x1a, 0x13, 0xbe, 0x20, 0xfe, 0x23, 0x7d, 0xfe,
                0x34, 0x20, 0xf5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xfb, 0x86, 0x20, 0xfe,
                0x3e, 0x01, 0xe0, 0x50,
            ],
        }
    }

    fn get_instruction(byte: &u8) -> (Instruction, u8) {
        match byte {
            0x00 => (Instruction::Nop, 1),
            0x10 => (Instruction::Stop, 2),
            0x01 | 0x11 | 0x21 | 0x31 => {
                let target = match byte {
                    0x01 => LoadTarget::BC,
                    0x11 => LoadTarget::DE,
                    0x21 => LoadTarget::HL,
                    0x31 => LoadTarget::SP,
                    _ => panic!("Unreachable Instruction"),
                };

                (Instruction::Load(target, LoadSource::PC16), 3)
            }
            0x02 | 0x12 | 0x22 | 0x32 => {
                let addr = match byte {
                    0x02 => LoadTarget::BCAddr,
                    0x12 => LoadTarget::DEAddr,
                    0x22 => LoadTarget::HLAddrInc,
                    0x32 => LoadTarget::HLAddrDec,
                    _ => panic!("Unreachable Instruction"),
                };
                (Instruction::Load(addr, LoadSource::A), 2)
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                let target = match byte {
                    0x03 => IncrementTarget::BC,
                    0x13 => IncrementTarget::DE,
                    0x23 => IncrementTarget::HL,
                    0x33 => IncrementTarget::SP,
                    _ => panic!("Unreachable Instruction"),
                };

                (Instruction::Increment(target), 2)
            }
            0x04 | 0x14 | 0x24 | 0x34 => {
                let target = match byte {
                    0x04 => IncrementTarget::B,
                    0x14 => IncrementTarget::D,
                    0x24 => IncrementTarget::H,
                    0x34 | _ => IncrementTarget::HLAddr,
                };
                let duration = if *byte == 0x34 { 2 } else { 1 };
                (Instruction::Increment(target), duration)
            }
            0x05 | 0x15 | 0x25 | 0x35 => {
                let target = match byte {
                    0x05 => DecrementTarget::B,
                    0x15 => DecrementTarget::D,
                    0x25 => DecrementTarget::H,
                    0x35 => DecrementTarget::HLAddr,
                    _ => panic!("Unreachable Instruction"),
                };
                let duration = if *byte == 0x35 { 2 } else { 1 };
                (Instruction::Decrement(target), duration)
            }
            0x06 | 0x16 | 0x26 | 0x36 => {
                let target = match byte {
                    0x06 => LoadTarget::B,
                    0x16 => LoadTarget::D,
                    0x26 => LoadTarget::H,
                    0x36 => LoadTarget::HL,
                    _ => panic!("Unreachable Instruction"),
                };
                let duration = if *byte == 0x36 { 3 } else { 2 };

                (Instruction::Load(target, LoadSource::PC), duration)
            }
            0x07 => (Instruction::RotateLeftCircular(BitwiseSource::A), 1),
            0x17 => (Instruction::RotateLeft(BitwiseSource::A), 1),
            0x27 => (Instruction::DecimalAdjustAccumulator, 1),
            0x37 => (Instruction::SetCarryFlag, 1),
            0x08 => (Instruction::Load(LoadTarget::PC16Addr, LoadSource::SP), 5),
            0x20 | 0x30 | 0x18 | 0x28 | 0x38 => {
                let condition = match byte {
                    0x20 => JumpCondition::NZ,
                    0x30 => JumpCondition::NC,
                    0x18 => JumpCondition::None,
                    0x28 => JumpCondition::Z,
                    0x38 => JumpCondition::C,
                    _ => panic!("Unreachable Instruction"),
                };
                (Instruction::JumpRelative(condition), 2)
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                let source = match byte {
                    0x09 => AddSource::BC,
                    0x19 => AddSource::DE,
                    0x29 => AddSource::HL,
                    0x39 => AddSource::SP,
                    _ => panic!("Unreachable Instruction"),
                };
                (Instruction::Add(AddTarget::HL, source), 2)
            }
            0x0a | 0x1a | 0x2a | 0x3a => {
                let source = match byte {
                    0x0a => LoadSource::BCAddr,
                    0x1a => LoadSource::DEAddr,
                    0x2a => LoadSource::HLAddrInc,
                    0x3a => LoadSource::HLAddrDec,
                    _ => panic!("Unreachable Instruction"),
                };

                (Instruction::Load(LoadTarget::A, source), 2)
            }
            0x0b | 0x1b | 0x2b | 0x3b => {
                let target = match byte {
                    0x0b => DecrementTarget::BC,
                    0x1b => DecrementTarget::DE,
                    0x2b => DecrementTarget::HL,
                    0x3b => DecrementTarget::SP,
                    _ => panic!("Unreachable Instruction"),
                };

                (Instruction::Decrement(target), 2)
            }
            0x0c | 0x1c | 0x2c | 0x3c => {
                let target = match byte {
                    0x0c => IncrementTarget::C,
                    0x1c => IncrementTarget::E,
                    0x2c => IncrementTarget::L,
                    0x3c => IncrementTarget::A,
                    _ => panic!("Unreachable Instruction"),
                };

                (Instruction::Increment(target), 1)
            }
            0x0d | 0x1d | 0x2d | 0x3d => {
                let target = match byte {
                    0x0d => DecrementTarget::C,
                    0x1d => DecrementTarget::E,
                    0x2d => DecrementTarget::L,
                    0x3d => DecrementTarget::A,
                    _ => panic!("Unreachable Instruction"),
                };

                (Instruction::Decrement(target), 1)
            }
            0x0e | 0x1e | 0x2e | 0x3e => {
                let target = match byte {
                    0x0e => LoadTarget::C,
                    0x1e => LoadTarget::E,
                    0x2e => LoadTarget::L,
                    0x3e => LoadTarget::A,
                    _ => panic!("Unreachable Instruction"),
                };

                (Instruction::Load(target, LoadSource::PC), 2)
            }
            0x0f => (Instruction::RotateRightCircular(BitwiseSource::A), 1),
            0x1f => (Instruction::RotateRight(BitwiseSource::A), 1),
            0x2f => (Instruction::ComplementAccumulator, 1),
            0x3f => (Instruction::ComplementCarryFlag, 1),
            0x40..=0x75 | 0x77..=0x7f => {
                let target = match byte {
                    0x40..=0x47 => LoadTarget::B,
                    0x48..=0x4f => LoadTarget::C,
                    0x50..=0x57 => LoadTarget::D,
                    0x58..=0x5f => LoadTarget::E,
                    0x60..=0x67 => LoadTarget::H,
                    0x68..=0x6f => LoadTarget::L,
                    0x70..=0x77 => LoadTarget::HLAddr,
                    0x78..=0x7f => LoadTarget::A,
                    _ => panic!("Unreachable Instruction"),
                };

                let source = match byte & 0x0F {
                    0x00 | 0x08 => LoadSource::B,
                    0x01 | 0x09 => LoadSource::C,
                    0x02 | 0x0a => LoadSource::D,
                    0x03 | 0x0b => LoadSource::E,
                    0x04 | 0x0c => LoadSource::H,
                    0x05 | 0x0d => LoadSource::L,
                    0x06 | 0x0e => LoadSource::HLAddr,
                    0x07 | 0x0f => LoadSource::A,

                    _ => panic!("Unreachable Instruction"),
                };

                let cycles = match byte {
                    0x70..=0x77 => 2,
                    _ if (byte & 0x0f) == 0x06 || (byte & 0x0f) == 0x0e => 2,
                    _ => 1,
                };

                (Instruction::Load(target, source), cycles)
            }
            0x76 => (Instruction::Halt, 1),
            0x80..=0x87 => {
                let source = match byte & 0x0f {
                    0x00 => AddSource::B,
                    0x01 => AddSource::C,
                    0x02 => AddSource::D,
                    0x03 => AddSource::E,
                    0x04 => AddSource::H,
                    0x05 => AddSource::L,
                    0x06 => AddSource::HLAddr,
                    0x07 => AddSource::A,
                    _ => panic!("Unreachable Instruction"),
                };

                let cycles = if source == AddSource::HLAddr { 2 } else { 1 };

                (Instruction::Add(AddTarget::A, source), cycles)
            }
            0x88..=0x8f => {
                let source = match byte & 0x0f {
                    0x08 => AddCarrySource::B,
                    0x09 => AddCarrySource::C,
                    0x0a => AddCarrySource::D,
                    0x0b => AddCarrySource::E,
                    0x0c => AddCarrySource::H,
                    0x0d => AddCarrySource::L,
                    0x0e => AddCarrySource::HLAddr,
                    0x0f => AddCarrySource::A,
                    _ => panic!("Unreachable Instruction"),
                };

                let cycles = if source == AddCarrySource::HLAddr {
                    2
                } else {
                    1
                };

                (Instruction::AddCarry(source), cycles)
            }
            0x90..=0x97 => {
                let source = match byte & 0x0f {
                    0x00 => SubtractSource::B,
                    0x01 => SubtractSource::C,
                    0x02 => SubtractSource::D,
                    0x03 => SubtractSource::E,
                    0x04 => SubtractSource::H,
                    0x05 => SubtractSource::L,
                    0x06 => SubtractSource::HLAddr,
                    0x07 => SubtractSource::A,
                    _ => panic!("Unreachable Instruction"),
                };

                let cycles = if source == SubtractSource::HLAddr {
                    2
                } else {
                    1
                };

                (Instruction::Subtract(source), cycles)
            }
            0x98..=0x9f => {
                let source = match byte & 0x0f {
                    0x08 => SubtractCarrySource::B,
                    0x09 => SubtractCarrySource::C,
                    0x0a => SubtractCarrySource::D,
                    0x0b => SubtractCarrySource::E,
                    0x0c => SubtractCarrySource::H,
                    0x0d => SubtractCarrySource::L,
                    0x0e => SubtractCarrySource::HLAddr,
                    0x0f => SubtractCarrySource::A,
                    _ => panic!("Unreachable Instruction"),
                };

                let cycles = if source == SubtractCarrySource::HLAddr {
                    2
                } else {
                    1
                };

                (Instruction::SubtractCarry(source), cycles)
            }
            0xa0..=0xa7 => {
                let source = match byte & 0x0f {
                    0x00 => AndSource::B,
                    0x01 => AndSource::C,
                    0x02 => AndSource::D,
                    0x03 => AndSource::E,
                    0x04 => AndSource::H,
                    0x05 => AndSource::L,
                    0x06 => AndSource::HLAddr,
                    0x07 => AndSource::A,
                    _ => panic!("Unreachable Instruction"),
                };

                let cycles = if source == AndSource::HLAddr { 2 } else { 1 };

                (Instruction::And(source), cycles)
            }
            0xa8..=0xaf => {
                let source = match byte & 0x0f {
                    0x08 => XOrSource::B,
                    0x09 => XOrSource::C,
                    0x0a => XOrSource::D,
                    0x0b => XOrSource::E,
                    0x0c => XOrSource::H,
                    0x0d => XOrSource::L,
                    0x0e => XOrSource::HLAddr,
                    0x0f => XOrSource::A,
                    _ => panic!("Unreachable Instruction"),
                };

                let cycles = if source == XOrSource::HLAddr { 2 } else { 1 };

                (Instruction::XOr(source), cycles)
            }
            0xb0..=0xb7 => {
                let source = match byte & 0x0f {
                    0x00 => OrSource::B,
                    0x01 => OrSource::C,
                    0x02 => OrSource::D,
                    0x03 => OrSource::E,
                    0x04 => OrSource::H,
                    0x05 => OrSource::L,
                    0x06 => OrSource::HLAddr,
                    0x07 => OrSource::A,
                    _ => panic!("Unreachable Instruction"),
                };

                let cycles = if source == OrSource::HLAddr { 2 } else { 1 };

                (Instruction::Or(source), cycles)
            }
            0xb8..=0xbf => {
                let source = match byte & 0x0f {
                    0x08 => CompareSource::B,
                    0x09 => CompareSource::C,
                    0x0a => CompareSource::D,
                    0x0b => CompareSource::E,
                    0x0c => CompareSource::H,
                    0x0d => CompareSource::L,
                    0x0e => CompareSource::HLAddr,
                    0x0f => CompareSource::A,
                    _ => panic!("Unreachable Instruction"),
                };

                let cycles = if source == CompareSource::HLAddr {
                    2
                } else {
                    1
                };

                (Instruction::Compare(source), cycles)
            }
            0xc0 => (Instruction::Return(JumpCondition::NZ), 2),
            0xd0 => (Instruction::Return(JumpCondition::NC), 2),
            0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                let target = match byte & 0xF0 {
                    0xc0 => PopTarget::BC,
                    0xd0 => PopTarget::DE,
                    0xe0 => PopTarget::HL,
                    0xf0 => PopTarget::AF,
                    _ => panic!("Unreachable Instruction"),
                };

                (Instruction::Pop(target), 3)
            }
            0xc5 | 0xd5 | 0xe5 | 0xf5 => {
                let target = match byte & 0xF0 {
                    0xc0 => PushTarget::BC,
                    0xd0 => PushTarget::DE,
                    0xe0 => PushTarget::HL,
                    0xf0 => PushTarget::AF,
                    _ => panic!("Unreachable Instruction"),
                };

                (Instruction::Push(target), 3)
            }
            0xc2 => (Instruction::Jump(JumpCondition::NZ), 3),
            0xd2 => (Instruction::Jump(JumpCondition::NC), 3),
            0xc3 => (Instruction::Jump(JumpCondition::None), 4),
            0xe0 => (
                Instruction::LoadAccumulator(
                    LoadAccumulatorTarget::PCAddr,
                    LoadAccumulatorSource::A,
                ),
                3,
            ),
            0xf0 => (
                Instruction::LoadAccumulator(
                    LoadAccumulatorTarget::A,
                    LoadAccumulatorSource::PCAddr,
                ),
                3,
            ),
            0xe2 => (
                Instruction::LoadAccumulator(
                    LoadAccumulatorTarget::CAddr,
                    LoadAccumulatorSource::A,
                ),
                2,
            ),
            0xf2 => (
                Instruction::LoadAccumulator(
                    LoadAccumulatorTarget::A,
                    LoadAccumulatorSource::CAddr,
                ),
                2,
            ),
            0xf3 => (Instruction::DisableInterrupts, 1),
            0xc4 => (Instruction::Call(JumpCondition::NZ), 3),
            0xd4 => (Instruction::Call(JumpCondition::NC), 3),
            0xc6 => (Instruction::Add(AddTarget::A, AddSource::PC), 2),
            0xd6 => (Instruction::Subtract(SubtractSource::PC), 2),
            0xe6 => (Instruction::And(AndSource::PC), 2),
            0xf6 => (Instruction::Or(OrSource::PC), 2),
            0xc7 => (Instruction::Restart(0x00), 4),
            0xd7 => (Instruction::Restart(0x10), 4),
            0xe7 => (Instruction::Restart(0x20), 4),
            0xf7 => (Instruction::Restart(0x30), 4),
            0xc8 => (Instruction::Return(JumpCondition::Z), 2),
            0xd8 => (Instruction::Return(JumpCondition::C), 2),
            0xe8 => (Instruction::Add(AddTarget::SP, AddSource::PCSigned), 4),
            0xf8 => (Instruction::Load(LoadTarget::HL, LoadSource::SPE), 3),
            0xc9 => (Instruction::Return(JumpCondition::None), 4),
            0xd9 => (Instruction::ReturnInterrupt, 4),
            0xe9 => (Instruction::JumpHL, 1),
            0xf9 => (Instruction::Load(LoadTarget::SP, LoadSource::HL), 2),
            0xca => (Instruction::Jump(JumpCondition::Z), 3),
            0xda => (Instruction::Jump(JumpCondition::C), 3),
            0xea => (Instruction::Load(LoadTarget::PCAddr, LoadSource::A), 4),
            0xfa => (Instruction::Load(LoadTarget::A, LoadSource::PCAddr), 4),
            0xcb => (Instruction::CB, 1),
            0xfb => (Instruction::EnableInterrupts, 1),
            0xcc => (Instruction::Call(JumpCondition::Z), 3),
            0xdc => (Instruction::Call(JumpCondition::C), 3),
            0xcd => (Instruction::Call(JumpCondition::C), 6),
            0xce => (Instruction::AddCarry(AddCarrySource::PC), 2),
            0xde => (Instruction::SubtractCarry(SubtractCarrySource::PC), 2),
            0xee => (Instruction::XOr(XOrSource::PC), 2),
            0xfe => (Instruction::Compare(CompareSource::PC), 2),
            0xcf => (Instruction::Restart(0x08), 4),
            0xdf => (Instruction::Restart(0x18), 4),
            0xef => (Instruction::Restart(0x28), 4),
            0xff => (Instruction::Restart(0x38), 4),
            0xd3 | 0xe3 | 0xe4 | 0xf4 | 0xdb | 0xeb | 0xec | 0xfc | 0xdd | 0xed | 0xfd => {
                panic!("Unknown Instruction Code: {:#x}", byte)
            }
        }
    }

    fn get_cb_instruction(byte: &u8) -> (Instruction, u8) {
        let source = match byte & 0x0F {
            0x00 | 0x08 => BitwiseSource::B,
            0x01 | 0x09 => BitwiseSource::C,
            0x02 | 0x0a => BitwiseSource::D,
            0x03 | 0x0b => BitwiseSource::E,
            0x04 | 0x0c => BitwiseSource::H,
            0x05 | 0x0d => BitwiseSource::L,
            0x06 | 0x0e => BitwiseSource::HLAddr,
            0x07 | 0x0f => BitwiseSource::A,
            _ => panic!("Unreachable Instruction"),
        };
        let cycles = if source == BitwiseSource::HLAddr {
            3
        } else {
            1
        };
        match byte {
            0x00..=0x07 => (Instruction::RotateLeftCircular(source), cycles),
            0x08..=0x0f => (Instruction::RotateRightCircular(source), cycles),
            0x10..=0x17 => (Instruction::RotateLeft(source), cycles),
            0x18..=0x1f => (Instruction::RotateRight(source), cycles),
            0x20..=0x27 => (Instruction::ShiftLeftArithmetic(source), cycles),
            0x28..=0x2f => (Instruction::ShiftRightArithmetic(source), cycles),
            0x30..=0x37 => (Instruction::Swap(source), cycles),
            0x38..=0x3f => (Instruction::ShiftRightLogical(source), cycles),
            0x40..=0x47 => (Instruction::Bit(0, source), cycles),
            0x48..=0x4f => (Instruction::Bit(1, source), cycles),
            0x50..=0x57 => (Instruction::Bit(2, source), cycles),
            0x58..=0x5f => (Instruction::Bit(3, source), cycles),
            0x60..=0x67 => (Instruction::Bit(4, source), cycles),
            0x68..=0x6f => (Instruction::Bit(5, source), cycles),
            0x70..=0x77 => (Instruction::Bit(6, source), cycles),
            0x78..=0x7f => (Instruction::Bit(7, source), cycles),
            _ => panic!("Unknown Instruction: {:#x}", byte),
        }
    }

    fn rotate_left(&mut self, source: BitwiseSource) {}

    fn write(&self, addr: &u16, value: u8) {
        println!("writing {:#x} to {:#x}", value, addr);
    }

    fn read(&self, addr: &u16) -> u8 {
        return match addr {
            0x0000..=0x00ff if !self.boot_off => self.boot_rom[*addr as usize],
            0x0000..=0x7fff | 0xA000..=0xBfff => self
                .cart
                .read(addr)
                .expect("failed to read from cart: {addr}"),
            0x8000..=0x9fff => self.v_ram[self.v_ram_bank as usize][(addr - 0x8000) as usize],
            0xC000..=0xCfff => self.w_ram[0][(addr - 0xC000) as usize],
            0xd000..=0xdfff => self.w_ram[self.w_ram_bank as usize][(addr - 0xd000) as usize],
            0xe000..=0xefff => self.w_ram[0][(addr - 0xe000) as usize],
            0xf000..=0xfdff => self.w_ram[self.w_ram_bank as usize][(addr - 0xf000) as usize],
            0xfea0..=0xfeff => {
                println!("accessing unusable memory: {}", addr);
                0xff
            }

            _ => panic!("unimplemented read at location: {:#x}", addr),
        };
    }

    fn jump_relative(&mut self, condition: JumpCondition) {
        if condition == JumpCondition::None {
            let e = self.read(&self.program_counter) as i8;
            self.program_counter += 1;
            self.program_counter = (self.program_counter as i16 + e as i16) as u16;
            println!("jump to: {:#x}", self.program_counter);
            return;
        }

        let condition = match condition {
            JumpCondition::None => unreachable!(),
            JumpCondition::Z => self.get_flag(Flag::Z),
            JumpCondition::C => self.get_flag(Flag::C),
            JumpCondition::NZ => !self.get_flag(Flag::Z),
            JumpCondition::NC => !self.get_flag(Flag::C),
        };

        if condition {
            self.instruction = Instruction::JumpRelative(JumpCondition::None);
            self.step_count = 1;
        } else {
            self.program_counter += 1;
        }
    }

    fn call(&mut self, condition: JumpCondition) {
        if condition == JumpCondition::None {
            let addr = self.read(&self.program_counter) as u16;
            self.program_counter += 1;
            let addr = addr | (self.read(&self.program_counter) as u16) << 8;
            self.program_counter += 1;

            self.stack_pointer -= 1;
            self.write(&self.stack_pointer, (self.program_counter >> 8) as u8);
            self.stack_pointer -= 1;
            self.write(&self.stack_pointer, self.program_counter as u8);

            self.program_counter = addr;
            return;
        }

        let condition = match condition {
            JumpCondition::None => unreachable!(),
            JumpCondition::Z => self.get_flag(Flag::Z),
            JumpCondition::C => self.get_flag(Flag::C),
            JumpCondition::NZ => !self.get_flag(Flag::Z),
            JumpCondition::NC => !self.get_flag(Flag::C),
        };

        if condition {
            self.instruction = Instruction::Call(JumpCondition::None);
            self.step_count = 3;
        } else {
            self.program_counter += 3;
        }
    }

    fn sub(&mut self, source: SubtractSource) {
        let source = match source {
            SubtractSource::A => self.register.get_a(),
            SubtractSource::B => self.register.get_b(),
            SubtractSource::C => self.register.get_c(),
            SubtractSource::D => self.register.get_d(),
            SubtractSource::E => self.register.get_e(),
            SubtractSource::H => self.register.get_h(),
            SubtractSource::L => self.register.get_l(),
            SubtractSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
            SubtractSource::HLAddr => self.read(&self.register.get_hl()),
        };
        let (result, c, h) = Cpu::subtract(&self.register.get_a(), &source);

        self.register.set_a(result);
        self.update_flag(Flag::C, c);
        self.update_flag(Flag::H, h);
        self.set_flag(Flag::N);
        self.update_flag(Flag::Z, result == 0);
    }

    fn bit(&mut self, bit: u8, source: BitwiseSource) {
        let source = match source {
            BitwiseSource::B => self.register.get_b(),
            BitwiseSource::C => self.register.get_c(),
            BitwiseSource::D => self.register.get_d(),
            BitwiseSource::E => self.register.get_e(),
            BitwiseSource::H => self.register.get_h(),
            BitwiseSource::L => self.register.get_l(),
            BitwiseSource::A => self.register.get_a(),
            BitwiseSource::HLAddr => self.read(&self.register.get_hl()),
        };

        self.update_flag(Flag::Z, ((source >> bit) & 1) == 0);
        self.set_flag(Flag::H);
        self.clear_flag(Flag::N);
    }

    fn compare(&mut self, source: CompareSource) {
        let value = match source {
            CompareSource::A => self.register.get_a(),
            CompareSource::B => self.register.get_b(),
            CompareSource::C => self.register.get_c(),
            CompareSource::D => self.register.get_d(),
            CompareSource::E => self.register.get_e(),
            CompareSource::H => self.register.get_h(),
            CompareSource::L => self.register.get_l(),
            CompareSource::HLAddr => todo!(),
            CompareSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
        };

        let (result, c, h) = Cpu::subtract(&self.register.get_a(), &value);
        println!(
            "Comparing: {} - {} = {}",
            self.register.get_a(),
            value,
            result
        );
        self.update_flag(Flag::Z, result == 0);
        self.update_flag(Flag::N, true);
        self.update_flag(Flag::H, h);
        self.update_flag(Flag::C, c);
    }

    fn load(&mut self, target: LoadTarget, source: LoadSource) {
        let source: u16 = match source {
            LoadSource::PCAddr => {
                let lsb = self.read(&self.program_counter);
                self.program_counter += 1;
                let msb = self.read(&self.program_counter);
                self.program_counter += 1;
                let n = (msb as u16) << 8 | lsb as u16;
                self.read(&n) as u16
            }
            LoadSource::HL => self.register.get_hl(),
            LoadSource::SPE => todo!(),
            LoadSource::BCAddr => self.read(&self.register.get_bc()) as u16,
            LoadSource::DEAddr => self.read(&self.register.get_de()) as u16,
            LoadSource::HLAddrInc => {
                let hl = self.register.get_hl();
                let n = self.read(&hl);
                self.register.set_hl(hl + 1);
                n as u16
            }
            LoadSource::HLAddrDec => {
                let hl = self.register.get_hl();
                let n = self.read(&hl);
                self.register.set_hl(hl - 1);
                n as u16
            }
            LoadSource::SP => self.stack_pointer,
            LoadSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n as u16
            }
            LoadSource::PC16 => {
                let lsb = self.read(&self.program_counter);
                self.program_counter += 1;
                let msb = self.read(&self.program_counter);
                self.program_counter += 1;
                (msb as u16) << 8 | lsb as u16
            }
            LoadSource::A => self.register.get_a() as u16,
            LoadSource::B => self.register.get_b() as u16,
            LoadSource::C => self.register.get_c() as u16,
            LoadSource::D => self.register.get_d() as u16,
            LoadSource::E => self.register.get_e() as u16,
            LoadSource::H => self.register.get_h() as u16,
            LoadSource::L => self.register.get_l() as u16,
            LoadSource::HLAddr => self.read(&self.register.get_hl()) as u16,
        };

        match target {
            LoadTarget::A => self.register.set_a(source as u8),
            LoadTarget::B => self.register.set_b(source as u8),
            LoadTarget::C => self.register.set_c(source as u8),
            LoadTarget::D => self.register.set_d(source as u8),
            LoadTarget::E => self.register.set_e(source as u8),
            LoadTarget::H => self.register.set_h(source as u8),
            LoadTarget::L => self.register.set_l(source as u8),
            LoadTarget::BC => self.register.set_bc(source),
            LoadTarget::DE => self.register.set_de(source),
            LoadTarget::HL => self.register.set_hl(source),
            LoadTarget::SP => self.stack_pointer = source,
            LoadTarget::BCAddr => self.write(&self.register.get_bc(), source as u8),
            LoadTarget::DEAddr => self.write(&self.register.get_de(), source as u8),
            LoadTarget::HLAddrInc => {
                let hl = self.register.get_hl();
                self.write(&hl, source as u8);
                self.register.set_hl(hl + 1);
            }
            LoadTarget::HLAddrDec => {
                let hl = self.register.get_hl();
                self.write(&hl, source as u8);
                self.register.set_hl(hl - 1);
            }
            LoadTarget::HLAddr => self.write(&self.register.get_hl(), source as u8),
            LoadTarget::PCAddr => {
                let lsb = self.read(&self.program_counter);
                self.program_counter += 1;
                let msb = self.read(&self.program_counter);
                self.program_counter += 1;
                let n = (msb as u16) << 8 | lsb as u16;
                self.write(&n, source as u8);
            }
            LoadTarget::PC16Addr => {
                let lsb = self.read(&self.program_counter);
                self.program_counter += 1;
                let msb = self.read(&self.program_counter);
                self.program_counter += 1;
                let n = (msb as u16) << 8 | lsb as u16;
                self.write(&n, source as u8);
                let n = n + 1;
                self.write(&n, (source >> 8) as u8)
            }
        };
    }

    fn load_accumulator(&mut self, target: LoadAccumulatorTarget, source: LoadAccumulatorSource) {
        let source = match source {
            LoadAccumulatorSource::A => self.register.get_a(),
            LoadAccumulatorSource::PCAddr => {
                let n = self.read(&self.program_counter) as u16;
                self.program_counter += 1;
                self.read(&(0xFF00 | n))
            }
            LoadAccumulatorSource::CAddr => self.read(&(0xFF00 | (self.register.get_c() as u16))),
        };

        match target {
            LoadAccumulatorTarget::PCAddr => {
                let n = 0xFF00 | self.read(&self.program_counter) as u16;
                self.program_counter += 1;
                self.write(&n, source);
            }
            LoadAccumulatorTarget::A => self.register.set_a(source),
            LoadAccumulatorTarget::CAddr => {
                let addr = ((0xFF as u16) << 8) | (self.register.get_c() as u16);
                self.write(&addr, source);
            }
        };
    }

    fn increment(&mut self, target: IncrementTarget) {
        match target {
            IncrementTarget::BC => {
                let (result, _) = self.register.get_bc().overflowing_add(1);
                self.register.set_bc(result);
            }
            IncrementTarget::DE => {
                let (result, _) = self.register.get_de().overflowing_add(1);
                self.register.set_de(result);
            }
            IncrementTarget::HL => todo!(),
            IncrementTarget::SP => todo!(),
            IncrementTarget::B => {
                let (results, _, h) = Cpu::addition(&self.register.get_b(), &1);
                self.register.set_b(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::D => {
                let (results, _, h) = Cpu::addition(&self.register.get_d(), &1);
                self.register.set_d(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::H => {
                let (results, _, h) = Cpu::addition(&self.register.get_h(), &1);
                self.register.set_h(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::C => {
                let (results, _, h) = Cpu::addition(&self.register.get_c(), &1);
                self.register.set_c(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::E => {
                let (results, _, h) = Cpu::addition(&self.register.get_e(), &1);
                self.register.set_e(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::L => {
                let (results, _, h) = Cpu::addition(&self.register.get_l(), &1);
                self.register.set_l(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::A => {
                let (results, _, h) = Cpu::addition(&self.register.get_a(), &1);
                self.register.set_a(results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
            IncrementTarget::HLAddr => {
                let addr = self.register.get_hl();
                let (results, _, h) = Cpu::addition(&self.read(&addr), &1);
                self.write(&addr, results);
                self.update_flag(Flag::Z, results == 0);
                self.clear_flag(Flag::N);
                self.update_flag(Flag::H, h);
            }
        }
    }

    fn xor(&mut self, source: XOrSource) {
        let source = match source {
            XOrSource::PC => {
                let n = self.read(&self.program_counter);
                self.program_counter += 1;
                n
            }
            XOrSource::A => self.register.get_a(),
            XOrSource::B => self.register.get_b(),
            XOrSource::C => self.register.get_c(),
            XOrSource::D => self.register.get_d(),
            XOrSource::E => self.register.get_e(),
            XOrSource::H => self.register.get_h(),
            XOrSource::L => self.register.get_l(),
            XOrSource::HLAddr => self.read(&self.register.get_hl()),
        };
        let xor = self.register.get_a() ^ source;

        let mut results = 0u8;
        if xor == 0 {
            results |= Flag::Z as u8;
        }
        self.register.set_a(xor);
        self.register.set_f(results);
    }

    fn push(&mut self, target: PushTarget) {
        let (msb, lsb) = match target {
            PushTarget::BC => (self.register.get_b(), self.register.get_c()),
            PushTarget::DE => (self.register.get_d(), self.register.get_e()),
            PushTarget::HL => (self.register.get_h(), self.register.get_l()),
            PushTarget::AF => (self.register.get_a(), self.register.get_f()),
        };
        self.stack_pointer -= 1;
        self.write(&self.stack_pointer, msb);
        self.stack_pointer -= 1;
        self.write(&self.stack_pointer, lsb);
    }

    fn update_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    fn set_flag(&mut self, flag: Flag) {
        self.register.set_f(self.register.get_f() | flag as u8);
    }

    fn clear_flag(&mut self, flag: Flag) {
        self.register.set_f(self.register.get_f() & !(flag as u8));
    }

    fn get_flag(&self, flag: Flag) -> bool {
        self.register.get_f() & flag as u8 == flag as u8
    }

    fn addition(a: &u8, b: &u8) -> (u8, bool, bool) {
        let (results, overflow) = a.overflowing_add(*b);

        (results, overflow, (results ^ a ^ b) & 0x10 == 0x10)
    }

    fn subtract(a: &u8, b: &u8) -> (u8, bool, bool) {
        let (results, overflow) = a.overflowing_sub(*b);

        (results, overflow, (results ^ a ^ b) & 0x10 != 0x10)
    }
}
