use super::*;
use ckb_vm::instructions::insts;
use ckb_vm::instructions::{Instruction, Itype};

// TODO Better name
pub fn r64_imm_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Itype::new_u(
            opcode,
            rd(instruction_bits),
            rs1(instruction_bits),
            x(instruction_bits, 20, 6, 0),
        )
        .0,
    )
}

pub fn roriw_builder(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Itype::new_u(
            insts::OP_SLLIUW,
            rd(instruction_bits),
            rs1(instruction_bits),
            x(instruction_bits, 20, 5, 0),
        )
        .0,
    )
}
