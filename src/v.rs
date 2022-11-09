use super::*;
use ckb_vm::instructions::insts;
use ckb_vm::instructions::{Instruction, Itype, Rtype, VItype, VVtype, VXtype};

fn vm(instruction_bits: u32) -> bool {
    instruction_bits & 0x2000000 != 0
}

pub fn vx_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        VXtype::new(
            opcode,
            rd(instruction_bits),
            rs1(instruction_bits),
            rs2(instruction_bits),
            vm(instruction_bits),
        )
        .0,
    )
}

pub fn vv_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        VVtype::new(
            opcode,
            rd(instruction_bits),
            rs1(instruction_bits),
            rs2(instruction_bits),
            vm(instruction_bits),
        )
        .0,
    )
}

pub fn vi_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        VItype::new(
            opcode,
            rd(instruction_bits),
            rs2(instruction_bits),
            x(instruction_bits, 15, 5, 0),
            vm(instruction_bits),
        )
        .0,
    )
}

pub fn vsetvli_builder(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Itype::new_u(
            insts::OP_VSETVLI,
            rd(instruction_bits),
            rs1(instruction_bits),
            x(instruction_bits, 20, 11, 0),
        )
        .0,
    )
}

pub fn vsetivli_builder(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Itype::new_u(
            insts::OP_VSETIVLI,
            rd(instruction_bits),
            rs1(instruction_bits),
            x(instruction_bits, 20, 10, 0),
        )
        .0,
    )
}

pub fn vsetvl_builder(
    instruction_bits: u32,
    _: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Rtype::new(
            insts::OP_VSETVL,
            rd(instruction_bits),
            rs1(instruction_bits),
            rs2(instruction_bits),
        )
        .0,
    )
}
