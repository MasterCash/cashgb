use std::fmt::Display;

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
    PCe,
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
    Halted,
    Stopped,
    Errored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    VBlank = 1 << 0,
    LCD = 1 << 1,
    Timer = 1 << 2,
    Serial = 1 << 3,
    Joypad = 1 << 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Z = 1 << 7,
    N = 1 << 6,
    H = 1 << 5,
    C = 1 << 4,
}

impl Instruction {
    pub fn get_instruction(byte: &u8) -> (Instruction, u8) {
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
                    0x34 => IncrementTarget::HLAddr,
                    _ => IncrementTarget::HLAddr,
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
            0xe8 => (Instruction::Add(AddTarget::SP, AddSource::E), 4),
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

    pub fn get_cb_instruction(byte: &u8) -> (Instruction, u8) {
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
}
