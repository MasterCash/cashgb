//! Checks how many M-cycles the decoder assigns every opcode against the reference table
//! (<https://gbdev.io/gb-opcodes/optables/>). Blargg's `instr_timing` ROM catches the same
//! mistakes, but this names the exact opcode.

use cash_gb::cpu::instructions::Instruction;

/// M-cycles for each unprefixed opcode, row by high nibble. Conditional jumps, calls and
/// returns use their not-taken time; the CPU adds the extra cycles when a branch is taken.
/// 0 marks the 11 illegal opcodes.
const MAIN_OPCODE_CYCLES: [u8; 256] = [
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, // 0x
    1, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 1x
    2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 2x
    2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 3x
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 4x
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 5x
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 6x
    2, 2, 2, 2, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, // 7x
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 8x
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 9x
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // Ax
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // Bx
    2, 3, 3, 4, 3, 4, 2, 4, 2, 4, 3, 1, 3, 6, 2, 4, // Cx
    2, 3, 3, 0, 3, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4, // Dx
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, // Ex
    3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4, // Fx
];

#[test]
fn main_opcodes_take_the_right_number_of_cycles() {
    let mut wrong = Vec::new();
    for opcode in 0..=255u8 {
        let expected = MAIN_OPCODE_CYCLES[opcode as usize];
        if expected == 0 {
            continue; // illegal opcode
        }
        let (instruction, cycles) = Instruction::get_instruction(&opcode)
            .unwrap_or_else(|error| panic!("{opcode:#04X} failed to decode: {error:?}"));
        if cycles != expected {
            wrong.push(format!(
                "{opcode:02X} {instruction}: {cycles}, expected {expected}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "wrong cycle counts:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn cb_opcodes_take_the_right_number_of_cycles() {
    let mut wrong = Vec::new();
    for opcode in 0..=255u8 {
        // Total including the 1-cycle CB prefix: 2 for a register, 3 for BIT n,(HL), which
        // only reads it, and 4 for the other (HL) instructions, which read and write it.
        let on_hl = opcode & 0x07 == 0x06;
        let is_bit = (0x40..=0x7F).contains(&opcode);
        let expected = match (on_hl, is_bit) {
            (false, _) => 2,
            (true, true) => 3,
            (true, false) => 4,
        };
        let (instruction, cycles) = Instruction::get_cb_instruction(&opcode);
        if cycles + 1 != expected {
            wrong.push(format!(
                "CB {opcode:02X} {instruction}: {}, expected {expected}",
                cycles + 1
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "wrong cycle counts:\n{}",
        wrong.join("\n")
    );
}
