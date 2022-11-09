use super::*;
use ckb_vm::instructions::{blank_instruction, i::nop, insts};
use ckb_vm::instructions::{Instruction, Itype, Rtype, Stype, Utype};
use ckb_vm_definitions::registers::SP;

// Notice the location of rs2 in RVC encoding is different from full encoding
#[inline(always)]
fn c_rs2(instruction_bits: u32) -> usize {
    x(instruction_bits, 2, 5, 0) as usize
}

// This function extract 3 bits from least_bit to form a register number,
// here since we are only using 3 bits, we can only reference the most popular
// used registers x8 - x15. In other words, a number of 0 extracted here means
// x8, 1 means x9, etc.
#[inline(always)]
fn compact_register_number(instruction_bits: u32, least_bit: usize) -> usize {
    x(instruction_bits, least_bit, 3, 0) as usize + 8
}

// [12]  => imm[5]
// [6:2] => imm[4:0]
fn immediate(instruction_bits: u32) -> i32 {
    (x(instruction_bits, 2, 5, 0) | xs(instruction_bits, 12, 1, 5)) as i32
}

// [12]  => imm[5]
// [6:2] => imm[4:0]
fn uimmediate(instruction_bits: u32) -> u32 {
    x(instruction_bits, 2, 5, 0) | x(instruction_bits, 12, 1, 5)
}

// [12:2] => imm[11|4|9:8|10|6|7|3:1|5]
fn j_immediate(instruction_bits: u32) -> i32 {
    (x(instruction_bits, 3, 3, 1)
        | x(instruction_bits, 11, 1, 4)
        | x(instruction_bits, 2, 1, 5)
        | x(instruction_bits, 7, 1, 6)
        | x(instruction_bits, 6, 1, 7)
        | x(instruction_bits, 9, 2, 8)
        | x(instruction_bits, 8, 1, 10)
        | xs(instruction_bits, 12, 1, 11)) as i32
}

// [12:10] => uimm[5:3]
// [6:5]   => uimm[7:6]
fn fld_uimmediate(instruction_bits: u32) -> u32 {
    x(instruction_bits, 10, 3, 3) | x(instruction_bits, 5, 2, 6)
}

// [10:12] => uimm[5:3]
// [5:6]   => uimm[2|6]
fn sw_uimmediate(instruction_bits: u32) -> u32 {
    x(instruction_bits, 6, 1, 2) | x(instruction_bits, 10, 3, 3) | x(instruction_bits, 5, 1, 6)
}

// [12]  => uimm[5]
// [6:2] => uimm[4:2|7:6]
fn lwsp_uimmediate(instruction_bits: u32) -> u32 {
    x(instruction_bits, 4, 3, 2) | x(instruction_bits, 12, 1, 5) | x(instruction_bits, 2, 2, 6)
}

// [12]  => uimm[5]
// [6:2] => uimm[4:3|8:6]
fn fldsp_uimmediate(instruction_bits: u32) -> u32 {
    x(instruction_bits, 5, 2, 3) | x(instruction_bits, 12, 1, 5) | x(instruction_bits, 2, 3, 6)
}

// [12:7] => uimm[5:3|8:6]
fn fsdsp_uimmediate(instruction_bits: u32) -> u32 {
    x(instruction_bits, 10, 3, 3) | x(instruction_bits, 7, 3, 6)
}

// [12:7] => uimm[5:2|7:6]
fn swsp_uimmediate(instruction_bits: u32) -> u32 {
    x(instruction_bits, 9, 4, 2) | x(instruction_bits, 7, 2, 6)
}

// [12:10] => imm[8|4:3]
// [6:2]   => imm[7:6|2:1|5]
fn b_immediate(instruction_bits: u32) -> i32 {
    (x(instruction_bits, 3, 2, 1)
        | x(instruction_bits, 10, 2, 3)
        | x(instruction_bits, 2, 1, 5)
        | x(instruction_bits, 5, 2, 6)
        | xs(instruction_bits, 12, 1, 8)) as i32
}

pub fn cadd_ebreak_jalr(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    let rd = rd(instruction_bits);
    let rs2 = c_rs2(instruction_bits);
    match (rd, rs2) {
        // C.EBREAK
        (0, 0) => Some(blank_instruction(insts::OP_EBREAK)),
        // C.JALR
        (rs1, 0) => Some(Itype::new_s(insts::OP_JALR, 1, rs1, 0).0),
        // C.ADD
        (rd, rs2) => {
            if rd != 0 {
                Some(Rtype::new(insts::OP_ADD, rd, rd, rs2).0)
            } else if config.version >= 1 {
                // HINTs
                Some(nop())
            } else {
                None
            }
        }
    }
}

pub fn caddi_cnop(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    // C.ADDI is only valid when (rd!=0 && nzimm!=0).
    // rd=0 encodes NOP, _the remaining code points_ with nzimm=0 encodes HINTS
    let nzimm = immediate(instruction_bits);
    let rd = rd(instruction_bits);
    match (rd, nzimm) {
        (0, 0) => None,
        (0, _) => {
            // C.NOP
            Some(nop())
        }
        (_, 0) => {
            // HINTS
            Some(nop())
        }
        _ => Some(Itype::new_s(insts::OP_ADDI, rd, rd, nzimm).0),
    }
}

pub fn caddi16sp(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    // C.ADDI16SP is only valid when nzimm!=0, code point with nzimm=0 is reserved
    let nzimm = x(instruction_bits, 6, 1, 4)
        | x(instruction_bits, 2, 1, 5)
        | x(instruction_bits, 5, 1, 6)
        | x(instruction_bits, 3, 2, 7)
        | xs(instruction_bits, 12, 1, 9);
    if nzimm != 0 {
        Some(Itype::new_s(insts::OP_ADDI, SP, SP, nzimm as i32).0)
    } else {
        // reserved
        None
    }
}

pub fn caddi4spn(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    // C.ADDI4SPN is only valid when nzuimm=0, the code points with nzuimm=0 are reserved
    let nzuimm = x(instruction_bits, 6, 1, 2)
        | x(instruction_bits, 5, 1, 3)
        | x(instruction_bits, 11, 2, 4)
        | x(instruction_bits, 7, 4, 6);
    if nzuimm != 0 {
        // C.ADDI4SPN
        Some(
            Itype::new_u(
                insts::OP_ADDI,
                compact_register_number(instruction_bits, 2),
                SP,
                nzuimm,
            )
            .0,
        )
    } else {
        // reserved
        None
    }
}

pub fn caddiw_jal(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // C.ADDIW and C.JAL share the same masks
    // but former is 64/128-only and latter is 32 only
    if config.rv32 {
        Some(Utype::new_s(insts::OP_JAL, 1, j_immediate(instruction_bits)).0)
    } else {
        // C.ADDIW is only valid with rd=0, code points with rd=0 is reserved.
        let rd = rd(instruction_bits);
        if rd != 0 {
            Some(Itype::new_s(insts::OP_ADDIW, rd, rd, immediate(instruction_bits)).0)
        } else {
            // reserved
            None
        }
    }
}

pub fn caddw(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // C.ADDW is 64/128 only.
    let rd = compact_register_number(instruction_bits, 7);
    if config.rv64 {
        return Some(
            Rtype::new(
                insts::OP_ADDW,
                rd,
                rd,
                compact_register_number(instruction_bits, 2),
            )
            .0,
        );
    }
    None
}

pub fn cand(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    let rd = compact_register_number(instruction_bits, 7);
    Some(
        Rtype::new(
            insts::OP_AND,
            rd,
            rd,
            compact_register_number(instruction_bits, 2),
        )
        .0,
    )
}

pub fn candi(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    let rd = compact_register_number(instruction_bits, 7);
    Some(Itype::new_s(insts::OP_ANDI, rd, rd, immediate(instruction_bits)).0)
}

pub fn cbeqz(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Stype::new_s(
            insts::OP_BEQ,
            b_immediate(instruction_bits),
            compact_register_number(instruction_bits, 7),
            0,
        )
        .0,
    )
}

pub fn cbnez(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Stype::new_s(
            insts::OP_BNE,
            b_immediate(instruction_bits),
            compact_register_number(instruction_bits, 7),
            0,
        )
        .0,
    )
}

pub fn cj(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(Utype::new_s(insts::OP_JAL, 0, j_immediate(instruction_bits)).0)
}

pub fn cjr_cmv(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // C.JR and C.MV share the same mask
    // C.JR is only valid when rs1!=0, whose code point is reserved
    // C.MV is only valid when rs2!=0, whose code point is C.JR
    // when rs2!=0, rd=0, the code point corrspond to HINTS
    let rs1 = rd(instruction_bits);
    let rs2 = c_rs2(instruction_bits);
    if rs2 == 0 {
        if rs1 != 0 {
            // C.JR
            Some(Itype::new_s(insts::OP_JALR, 0, rs1, 0).0)
        } else {
            // Reserved
            None
        }
    } else {
        if rs1 != 0 {
            // C.MV
            Some(Rtype::new(insts::OP_ADD, rs1, 0, rs2).0)
        } else if config.version >= 1 {
            // HINTS
            Some(nop())
        } else {
            None
        }
    }
}

pub fn cld(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // c.ld is 64/128 only
    if config.rv32 {
        return None;
    }
    // C.LD
    Some(
        Itype::new_u(
            insts::OP_LD,
            compact_register_number(instruction_bits, 2),
            compact_register_number(instruction_bits, 7),
            fld_uimmediate(instruction_bits),
        )
        .0,
    )
}

pub fn cldsp(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // C.LDSP is 64/128 only, and only valid when rd!=0.
    if config.rv32 {
        return None;
    }
    let rd = rd(instruction_bits);
    if rd != 0 {
        Some(Itype::new_u(insts::OP_LD, rd, SP, fldsp_uimmediate(instruction_bits)).0)
    } else {
        // Reserved
        None
    }
}

pub fn cli(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // C.LI is only valid when rd!=0, whose code points otherwise are HINTs

    let rd = rd(instruction_bits);
    if rd != 0 {
        // C.LI
        Some(Itype::new_s(insts::OP_ADDI, rd, 0, immediate(instruction_bits)).0)
    } else if config.version >= 1 {
        // HINTs
        Some(nop())
    } else {
        None
    }
}

pub fn clui(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // C.LUI is only valid when rd!={0,2} and immediate != 0
    // code points with nzimm = 0 are reserved
    // the remaining code points with rd=0 are HINTs
    // the remaining code points with rd=2 are C.ADDI16SP
    let imm = immediate(instruction_bits) << 12;
    if imm == 0 {
        // reserved
        return None;
    }
    let rd = rd(instruction_bits);
    if rd != 0 {
        Some(Utype::new_s(insts::OP_LUI, rd, imm).0)
    } else if config.version > 1 {
        // HINTS
        Some(nop())
    } else {
        panic!("This cannot happen as CLUI and CADDI16SP should have different mask")
    }
}

pub fn clw(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    // C.LW
    Some(
        Itype::new_u(
            insts::OP_LW,
            compact_register_number(instruction_bits, 2),
            compact_register_number(instruction_bits, 7),
            sw_uimmediate(instruction_bits),
        )
        .0,
    )
}

pub fn clwsp(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    // C.LWSP is only valid when rd!=0, in which case the code points are reserved
    let rd = rd(instruction_bits);
    if rd != 0 {
        // C.LWSP
        Some(Itype::new_u(insts::OP_LW, rd, SP, lwsp_uimmediate(instruction_bits)).0)
    } else {
        // Reserved
        None
    }
}

pub fn cor(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    let rd = compact_register_number(instruction_bits, 7);
    Some(
        Rtype::new(
            insts::OP_OR,
            rd,
            rd,
            compact_register_number(instruction_bits, 2),
        )
        .0,
    )
}

pub fn csd(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // C.SD is 64/128 only
    if config.rv32 {
        None
    } else {
        Some(
            Stype::new_u(
                insts::OP_SD,
                fld_uimmediate(instruction_bits),
                compact_register_number(instruction_bits, 7),
                compact_register_number(instruction_bits, 2),
            )
            .0,
        )
    }
}

pub fn csdsp(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // C.SDSP is 64/128 only
    if config.rv32 {
        None
    } else {
        Some(
            Stype::new_u(
                insts::OP_SD,
                fsdsp_uimmediate(instruction_bits),
                2,
                c_rs2(instruction_bits),
            )
            .0,
        )
    }
}

pub fn cslli(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // TODO not sure if the current implementation respect the spec.
    // E.g., "For RV32C, shamt[5] must be non-zero, shamt[5]=1 are reserved"

    let uimm = uimmediate(instruction_bits);
    let rd = rd(instruction_bits);
    if rd != 0 && uimm != 0 {
        // C.SLLI
        Some(Itype::new_u(insts::OP_SLLI, rd, rd, uimm & u32::from(config.shift_masks)).0)
    } else if config.version >= 1 {
        // HINTs
        Some(nop())
    } else {
        None
    }
}

pub fn csrai(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    let rd = compact_register_number(instruction_bits, 7);
    let uimm = uimmediate(instruction_bits);
    Some(Itype::new_u(insts::OP_SRAI, rd, rd, uimm & u32::from(config.shift_masks)).0)
}

pub fn csrli(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    let rd = compact_register_number(instruction_bits, 7);
    let uimm = uimmediate(instruction_bits);
    Some(Itype::new_u(insts::OP_SRLI, rd, rd, uimm & u32::from(config.shift_masks)).0)
}

pub fn csub(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    let rd = compact_register_number(instruction_bits, 7);
    Some(
        Rtype::new(
            insts::OP_SUB,
            rd,
            rd,
            compact_register_number(instruction_bits, 2),
        )
        .0,
    )
}

pub fn csubw(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    // C.SUBW is 64/128 only
    let rd = compact_register_number(instruction_bits, 7);
    if config.rv32 {
        return None;
    }
    Some(
        Rtype::new(
            insts::OP_SUBW,
            rd,
            rd,
            compact_register_number(instruction_bits, 2),
        )
        .0,
    )
}

pub fn csw(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Stype::new_u(
            insts::OP_SW,
            sw_uimmediate(instruction_bits),
            compact_register_number(instruction_bits, 7),
            compact_register_number(instruction_bits, 2),
        )
        .0,
    )
}

pub fn cswsp(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        // C.SWSP
        Stype::new_u(
            insts::OP_SW,
            swsp_uimmediate(instruction_bits),
            SP,
            c_rs2(instruction_bits),
        )
        .0,
    )
}

pub fn cxor(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    let rd = compact_register_number(instruction_bits, 7);
    Some(
        Rtype::new(
            insts::OP_XOR,
            rd,
            rd,
            compact_register_number(instruction_bits, 2),
        )
        .0,
    )
}
