include!("./mod.rs");
pub mod b;
pub mod common;
pub mod i;
// pub mod c;
pub mod v;

use b::*;
use common::*;
use i::*;
// use c::*;
use ckb_vm::instructions::insts;
use v::*;

use ckb_vm::instructions::Instruction;
use ckb_vm::Register;


pub type OpcodeBuilder = fn(
    instruction_bits: u32,
    insts::InstructionOpcode,
    config: &FactoryConfig,
) -> Option<Instruction>;


// TODO solve this, can we be generic?
// To be generic, means the instruction_lists needs to be initilized at runtime
// This is because function pointer cannot take generic type.
// This means overhead at startup for each factory, but can we be lazy_static?
pub struct FactoryConfig {
    rv32: bool,
    rv64: bool,
    version: u32,
    shift_masks: u8,
}

impl FactoryConfig {
    pub const fn new<R: Register>(version: u32) -> Self {
        Self {
            rv32: R::BITS == 32,
            rv64: R::BITS == 64,
            version,
            shift_masks: R::SHIFT_MASK,
        }
    }
}

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
