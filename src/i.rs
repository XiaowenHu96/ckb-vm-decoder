// TODO xiaowen: quick hack, fix API
// The FENCE instruction is used to order device I/O and memory accesses
// as viewed by other RISC- V harts and external devices or coprocessors.

use super::*;
use ckb_vm::instructions::insts;
use ckb_vm::instructions::{Instruction, Itype, Rtype};

#[derive(Debug, Clone, Copy)]
pub struct FenceType(Instruction);

impl FenceType {
    pub fn new(fm: u8, pred: u8, succ: u8) -> Self {
        FenceType(Rtype::new(insts::OP_FENCE, fm as usize, pred as usize, succ as usize).0)
    }

    pub fn fm(self) -> u8 {
        Rtype(self.0).rd() as u8
    }

    pub fn pred(self) -> u8 {
        Rtype(self.0).rs1() as u8
    }

    pub fn succ(self) -> u8 {
        Rtype(self.0).rs2() as u8
    }
}

pub fn is_alu_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Itype::new_s(
            opcode,
            rd(instruction_bits),
            rs1(instruction_bits),
            itype_immediate(instruction_bits) & i32::from(config.shift_masks),
        )
        .0,
    )
}
pub fn is_1f_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Itype::new_s(
            opcode,
            rd(instruction_bits),
            rs1(instruction_bits),
            itype_immediate(instruction_bits) & 0x1F,
        )
        .0,
    )
}

pub fn fencei_builder(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        FenceType::new(
            ((instruction_bits & 0xF00_00000) >> 28) as u8,
            ((instruction_bits & 0x0F0_00000) >> 24) as u8,
            ((instruction_bits & 0x00F_00000) >> 20) as u8,
        )
        .0,
    )
}
