include!("./mod.rs");
use ckb_vm::instructions::insts;

use ckb_vm::instructions::{Instruction, Itype, Rtype, VItype, VVtype, VXtype};

// TODO Xiaowen: quick hack
#[inline(always)]
pub fn x(instruction_bits: u32, lower: usize, length: usize, shifts: usize) -> u32 {
    ((instruction_bits >> lower) & ((1 << length) - 1)) << shifts
}

#[inline(always)]
pub fn funct3(instruction_bits: u32) -> u32 {
    x(instruction_bits, 12, 3, 0)
}

#[inline(always)]
pub fn funct7(instruction_bits: u32) -> u32 {
    x(instruction_bits, 25, 7, 0)
}

#[inline(always)]
pub fn rd(instruction_bits: u32) -> usize {
    x(instruction_bits, 7, 5, 0) as usize
}

#[inline(always)]
pub fn rs1(instruction_bits: u32) -> usize {
    x(instruction_bits, 15, 5, 0) as usize
}

#[inline(always)]
pub fn rs2(instruction_bits: u32) -> usize {
    x(instruction_bits, 20, 5, 0) as usize
}

fn vm(instruction_bits: u32) -> bool {
    instruction_bits & 0x2000000 != 0
}

pub fn vx_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    VXtype::new(
        opcode,
        rd(instruction_bits),
        rs1(instruction_bits),
        rs2(instruction_bits),
        vm(instruction_bits),
    )
    .0
}

pub fn vv_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    VVtype::new(
        opcode,
        rd(instruction_bits),
        rs1(instruction_bits),
        rs2(instruction_bits),
        vm(instruction_bits),
    )
    .0
}

pub fn vi_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    VItype::new(
        opcode,
        rd(instruction_bits),
        rs2(instruction_bits),
        x(instruction_bits, 15, 5, 0),
        vm(instruction_bits),
    )
    .0
}

pub fn vsetvli_builder(instruction_bits: u32, _: insts::InstructionOpcode) -> Instruction {
    Itype::new_u(
        insts::OP_VSETVLI,
        rd(instruction_bits),
        rs1(instruction_bits),
        x(instruction_bits, 20, 11, 0),
    )
    .0
}

pub fn vsetivli_builder(instruction_bits: u32, _: insts::InstructionOpcode) -> Instruction {
    Itype::new_u(
        insts::OP_VSETIVLI,
        rd(instruction_bits),
        rs1(instruction_bits),
        x(instruction_bits, 20, 10, 0),
    )
    .0
}

pub fn vsetvl_builder(instruction_bits: u32, _: insts::InstructionOpcode) -> Instruction {
    Rtype::new(
        insts::OP_VSETVL,
        rd(instruction_bits),
        rs1(instruction_bits),
        rs2(instruction_bits),
    )
    .0
}
