include!("./mod.rs");
use ckb_vm::instructions::insts;

use ckb_vm::instructions::{
    blank_instruction, Instruction, Itype, Rtype, Stype, Utype, VItype, VVtype, VXtype,
};

// TODO xiaowen: quick hack, fix API
// The FENCE instruction is used to order device I/O and memory accesses
// as viewed by other RISC- V harts and external devices or coprocessors.
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

pub type OpcodeBuilder = fn(instruction_bits: u32, insts::InstructionOpcode) -> Instruction;

pub struct InstructionInfo {
    mask: u32,
    match_bits: u32,
    opcode: insts::InstructionOpcode,
    builder: OpcodeBuilder,
}

impl InstructionInfo {
    pub const fn new(
        mask: u32,
        match_bits: u32,
        opcode: insts::InstructionOpcode,
        builder: OpcodeBuilder,
    ) -> Self {
        InstructionInfo {
            mask,
            match_bits,
            opcode,
            builder,
        }
    }

    pub const fn get_match_bits(&self) -> u32 {
        self.match_bits
    }
}

// TODO Xiaowen: quick hack
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

fn vm(instruction_bits: u32) -> bool {
    instruction_bits & 0x2000000 != 0
}

pub fn us_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    Utype::new_s(
        opcode,
        rd(instruction_bits),
        utype_immediate(instruction_bits),
    )
    .0
}

pub fn is_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    Itype::new_s(
        opcode,
        rd(instruction_bits),
        rs1(instruction_bits),
        itype_immediate(instruction_bits),
    )
    .0
}

pub fn is_alu_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    Itype::new_s(
        opcode,
        rd(instruction_bits),
        rs1(instruction_bits),
        //TODO xiaowen: Just to compile, fix this
        itype_immediate(instruction_bits) & i32::from(10),
    )
    .0
}
pub fn is_1f_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    Itype::new_s(
        opcode,
        rd(instruction_bits),
        rs1(instruction_bits),
        itype_immediate(instruction_bits) & 0x1F,
    )
    .0
}

pub fn sb_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    Stype::new_s(
        opcode,
        btype_immediate(instruction_bits),
        rs1(instruction_bits),
        rs2(instruction_bits),
    )
    .0
}

pub fn ss_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    Stype::new_s(
        opcode,
        stype_immediate(instruction_bits),
        rs1(instruction_bits),
        rs2(instruction_bits),
    )
    .0
}

pub fn blank_inst_builder(_: u32, opcode: insts::InstructionOpcode) -> Instruction {
    blank_instruction(opcode)
}

pub fn fencei_builder(instruction_bits: u32, _: insts::InstructionOpcode) -> Instruction {
    FenceType::new(
        ((instruction_bits & 0xF00_00000) >> 28) as u8,
        ((instruction_bits & 0x0F0_00000) >> 24) as u8,
        ((instruction_bits & 0x00F_00000) >> 20) as u8,
    )
    .0
}

pub fn r_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    Rtype::new(
        opcode,
        rd(instruction_bits),
        rs1(instruction_bits),
        rs2(instruction_bits),
    )
    .0
}

// TODO Better name
pub fn r64_imm_builder(instruction_bits: u32, opcode: insts::InstructionOpcode) -> Instruction {
    Itype::new_u(
        opcode,
        rd(instruction_bits),
        rs1(instruction_bits),
        x(instruction_bits, 20, 6, 0),
    )
    .0
}

pub fn roriw_builder(instruction_bits: u32, _: insts::InstructionOpcode) -> Instruction {
    Itype::new_u(
        insts::OP_SLLIUW,
        rd(instruction_bits),
        rs1(instruction_bits),
        x(instruction_bits, 20, 5, 0),
    )
    .0
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
