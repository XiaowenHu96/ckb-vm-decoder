// Common utilities used among different instruction sets
// TODO Xiaowen: quick hack
use ckb_vm::instructions::{blank_instruction, Instruction, Itype, Rtype, Stype, Utype};
use ckb_vm_definitions::instructions as insts;
use super::FactoryConfig;

#[inline(always)]
pub fn x(instruction_bits: u32, lower: usize, length: usize, shifts: usize) -> u32 {
    ((instruction_bits >> lower) & ((1 << length) - 1)) << shifts
}

#[inline(always)]
pub fn xs(instruction_bits: u32, lower: usize, length: usize, shifts: usize) -> u32 {
    ((instruction_bits as i32) << (32 - lower - length) >> (32 - length) << shifts) as u32
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

#[inline(always)]
pub fn utype_immediate(instruction_bits: u32) -> i32 {
    xs(instruction_bits, 12, 20, 12) as i32
}

#[inline(always)]
pub fn itype_immediate(instruction_bits: u32) -> i32 {
    xs(instruction_bits, 20, 12, 0) as i32
}

#[inline(always)]
pub fn btype_immediate(instruction_bits: u32) -> i32 {
    (x(instruction_bits, 8, 4, 1)
        | x(instruction_bits, 25, 6, 5)
        | x(instruction_bits, 7, 1, 11)
        | xs(instruction_bits, 31, 1, 12)) as i32
}

#[inline(always)]
pub fn stype_immediate(instruction_bits: u32) -> i32 {
    (x(instruction_bits, 7, 5, 0) | xs(instruction_bits, 25, 7, 5)) as i32
}

pub fn us_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Utype::new_s(
            opcode,
            rd(instruction_bits),
            utype_immediate(instruction_bits),
        )
        .0,
    )
}

pub fn is_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Itype::new_s(
            opcode,
            rd(instruction_bits),
            rs1(instruction_bits),
            itype_immediate(instruction_bits),
        )
        .0,
    )
}

pub fn sb_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Stype::new_s(
            opcode,
            btype_immediate(instruction_bits),
            rs1(instruction_bits),
            rs2(instruction_bits),
        )
        .0,
    )
}

pub fn ss_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Stype::new_s(
            opcode,
            stype_immediate(instruction_bits),
            rs1(instruction_bits),
            rs2(instruction_bits),
        )
        .0,
    )
}

pub fn blank_inst_builder(
    _: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(blank_instruction(opcode))
}

pub fn r_builder(
    instruction_bits: u32,
    opcode: insts::InstructionOpcode,
    _: &FactoryConfig,
) -> Option<Instruction> {
    Some(
        Rtype::new(
            opcode,
            rd(instruction_bits),
            rs1(instruction_bits),
            rs2(instruction_bits),
        )
        .0,
    )
}
