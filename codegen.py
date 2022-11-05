#!/usr/bin/env python

import sys
import csv
import random
import string
sys.path.append("perfect-hash/")
from perfect_hash import generate_hash

class Instruction(object):
    def __init__(self, name, mask, match_bits, handler):
        self.name = name
        self.mask = mask
        self.match_bits = match_bits
        self.handler = handler

    def to_rust_construct_inst(self):
        return f"const INST_{self.name} : VInstruction = "\
        f"VInstruction::new({self.mask},{self.match_bits},"\
        f"insts::OP_{self.name},{self.handler});\n"

    def to_rust_inst_name(self):
        return f"INST_{self.name}"

"""
Prelude
"""
def prelude():
    return """
use ckb_vm_definitions::instructions as insts;

use super::utils::{self, rd, rs1, rs2};
use super::{
    instruction_opcode_name, set_instruction_length_4, v, Instruction, Itype, Register, Rtype,
    VItype, VVtype, VXtype,
};

//TODO review this
fn vm(instruction_bits: u32) -> bool {
    instruction_bits & 0x2000000 != 0
}

pub type OpcodeBuilder = fn(instruction_bits: u32, insts::InstructionOpcode) -> Instruction;

pub struct VInstruction {
    mask: u32,
    pub match_bits: u32,
    opcode: insts::InstructionOpcode,
    builder: OpcodeBuilder,
}

impl VInstruction {
    pub const fn new(
        mask: u32,
        match_bits: u32,
        opcode: insts::InstructionOpcode,
        builder: OpcodeBuilder,
    ) -> Self {
        VInstruction {
            mask,
            match_bits,
            opcode,
            builder,
        }
    }
}
"""

"""
Interface template
"""
def interface_template():
    return """
pub fn factory<R: Register>(instruction_bits: u32, _: u32) -> Option<Instruction> {{
    for mask in &MASKS {{
        let match_bits = instruction_bits & mask;
        let idx = find_{basename}(match_bits);
        if  idx < {size} && (INSTRUTION_LIST[idx].mask & instruction_bits) == INSTRUTION_LIST[idx].match_bits {
            return Some(set_instruction_length_4(
            (INSTRUTION_LIST[idx].builder)(instruction_bits,INSTRUTION_LIST[idx].opcode)))
        }}
    }}
    return None
}}
"""

"""
Generate postlude, i.e., instruction definitions and lists
"""
def postlude(instructions):
    code = ""
    # init each instruction
    for inst in instructions:
        code += inst.to_rust_construct_inst()
    # put all instruction in the list
    code += f"pub const INSTRUTION_LIST : [VInstruction; {len(instructions)}] = [\n"
    for inst in instructions:
        code += inst.to_rust_inst_name() + ",\n"
    code += "];\n"

    # find all mask, sort by its element size
    by_masks = dict()
    for inst in instructions:
        if inst.mask in by_masks:
            by_masks[inst.mask].append(inst)
        else:
            by_masks[inst.mask] = [inst]
    sorted_masks = [k for k in sorted(by_masks, key=lambda x: len(by_masks[x]), reverse=True)]
    code += f"const MASKS : [u32; {len(sorted_masks)}] =[\n"
    for mask in sorted_masks:
        code += mask + ",\n"
    code += "];\n"
    return code

class MyHash(object):
    # TODO: Review this
    def __init__(self, N):
        self.N = N
        self.salt = []

    def __call__(self, key):
        key = int(key, 16)
        while len(self.salt) != 4:
            self.salt.append(random.randint(1, self.N - 1))
        keys = [(key >> (8 * x) & 0xff) for x in range(0, 4)]
        return sum(self.salt[i] * (c)
                   for i, c in enumerate(keys)) % self.N
    
    header_template =  """
// --- ------------------------------------------- ---
// --- Following code is generated by perfect_hash ---
// --- ------------------------------------------- ---
const G_{BASENAME} : [u32;{size}] = $G;
"""
#TODO  review s0 +1
    hash_template= """
#[inline(always)]
fn hash_f_{basename}_{version}(key: u32) -> usize {{
    return ((key & 0xff) * {s0} + (key >> 8 & 0xff) * {s1} + (key >> 16 & 0xff) * {s2} + (key >> 24) * {s3}) as usize;
}}
    """

    hash_entry_template = """
#[inline(always)]
fn find_{basename}(key:u32) -> usize {{
    return ((G_{BASENAME}[hash_f_{basename}_1(key) % $NG] 
        + G_{BASENAME}[hash_f_{basename}_2(key) % $NG]) % $NG) as usize;
}}
"""

    test_template = """
const K_{BASENAME} : [u32;{size}] = {keys}
pub fn test_{basename}() {{
    for i in 0..{size} {{
        assert!(find_{basename}(K_{BASENAME}[i]) == i)
    }}
}}
"""

"""
File format:
instruction_name,mask,match_bits,handler
"""
def parse_key(filename):
    instructions = []
    with open(filename) as csvfile:
        reader = csv.reader(csvfile, delimiter=",")
        for line in reader:
            line = list(map(lambda x : str.strip(x), line))
            instructions.append(Instruction(line[0],line[1],line[2],line[3]))
    return instructions

"""
Generate code for a hashmap filter 
"""
def gen_hashmap_filter(basename, instructions):
    f1, f2, G = generate_hash([x.match_bits for x in instructions], MyHash)
    assert f1.N == f2.N == len(G)
    try:
        salt_len = len(f1.salt)
        assert salt_len == len(f2.salt)
    except TypeError:
        salt_len = None
    template = MyHash.header_template.format(BASENAME=str.upper(basename), size=len(G))
    # generate hash function 1
    template += MyHash.hash_template.format(basename=basename,version=1,
            s0=f1.salt[0], s1=f1.salt[1],s2=f1.salt[2],s3=f1.salt[3])
    # generate hash function 2
    template += MyHash.hash_template.format(basename=basename,version=2,
            s0=f2.salt[0], s1=f2.salt[1],s2=f2.salt[2],s3=f2.salt[3])
    # generate perfect hash
    template += MyHash.hash_entry_template.format(basename=basename, BASENAME=str.upper(basename))
    return string.Template(template).substitute(
        NS = salt_len,
        NG = len(G),
        G  = G,
    )
    

"""
Generate hashmaps based on the masks in instructions
Returns a list of hashmaps
"""
def build_hashmaps(instructions):
    basename = "rvv"
    code = gen_hashmap_filter(basename, instructions)
    # generate interface
    # generate code to iterate through each mask
    code += interface_template().format(basename=basename, size=len(instructions))
    # for mask, insts in by_masks.items():
    #     code += gen_hashmap_filter(mask, insts)
    # generate interface
    return code


def main():
    insts = parse_key("rv_v")
    code = prelude()
    code += build_hashmaps(insts)
    code += postlude(insts)
    keys = [x.match_bits for x in insts]
    keys_list = "["
    for k in keys:
        keys_list += k + ",\n"
    keys_list += "];"
    code += MyHash.test_template.format(BASENAME="RVV",basename="rvv",keys=keys_list, size=len(insts))
    print(code)
    

if __name__ == '__main__':
    main()