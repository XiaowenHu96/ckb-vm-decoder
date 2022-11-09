#!/usr/bin/env python

### TODO: Handle duplicate key by merging operations
### TODO: Add comment for duplicated key
### TODO: FIX API

import sys
import csv
import random
import string
import os.path
from optparse import OptionParser
from functools import reduce
sys.path.append("perfect-hash/")
from perfect_hash import generate_hash

SET_INSTRUCTION_LEN = ""

class Instruction(object):
    def __init__(self, name, mask, match_bits, handler, opcode_name=None):
        self.name = name
        self.mask = mask
        self.match_bits = match_bits
        self.handler = handler
        self.opcode_name = name
        if opcode_name is not None:
            self.opcode_name = opcode_name

    def to_rust_construct_inst(self):
        return f"const INST_{self.name} : InstructionInfo = "\
        f"InstructionInfo::new({self.mask},{self.match_bits},"\
        f"insts::OP_{self.opcode_name},{self.handler});\n"

    def to_rust_inst_name(self):
        return f"INST_{self.name}"

    def __str__(self):
        return f"{self.name},{self.mask},{self.match_bits},{self.handler}"

"""
Prelude
"""
def prelude():
    return f"""use super::*;
use ckb_vm::instructions::insts as insts;
use ckb_vm::Register;
use ckb_vm::instructions::{{{SET_INSTRUCTION_LEN}, Instruction}};
"""

"""
Interface template
"""
def interface_template():
    return """
pub fn factory<R: Register>(instruction_bits: u32, version: u32) -> Option<Instruction> {{
    let config = FactoryConfig::new::<R>(version);
    for mask in &MASKS {{
        let match_bits = instruction_bits & mask;
        let idx = find_{basename}(match_bits);
        if  idx < {size} && (INSTRUCTION_LIST[idx].mask & instruction_bits) == INSTRUCTION_LIST[idx].match_bits {{
            if let Some(instruction) = (INSTRUCTION_LIST[idx].builder)(instruction_bits,INSTRUCTION_LIST[idx].opcode, &config) {{
                return Some({SET_INSTRUCTION_LEN}(instruction));
            }} else {{
                return None;
            }}
        }}
    }}
    return None
}}
"""

"""
Generate postlude, i.e., instruction definitions and lists
"""
def postlude(basename, instructions):
    code = ""
    # init each instruction
    for inst in instructions:
        code += inst.to_rust_construct_inst()
    # put all instruction in the list
    code += f"pub const INSTRUCTION_LIST : [InstructionInfo; {len(instructions)}] = [\n"
    for inst in instructions:
        code += inst.to_rust_inst_name() + ",\n"
    code += "];\n"

    # find all mask, sort by its element size
    by_masks = get_by_mask(instructions)
    sorted_masks = [k for k in sorted(by_masks, key=lambda x: len(by_masks[x]), reverse=True)]
    code += f"const MASKS : [u32; {len(sorted_masks)}] =[\n"
    for mask in sorted_masks:
        code += mask + ",\n"
    code += "];\n"
    return code

"""
Generate test case
"""
def gen_test(basename, instructions):
    keys = [x.match_bits for x in instructions]
    keys_list = "["
    for k in keys:
        keys_list += k + ",\n"
    keys_list += "];"
    ret = MyHash.test_template.format(BASENAME=str.upper(basename),basename=basename,keys=keys_list, size=len(instructions))
    return ret

class MyHash(object):
    def __init__(self, N):
        self.N = N
        self.salt = []

    def __call__(self, key):
        key = int(key, 16)
        while len(self.salt) != 4:
            self.salt.append(random.randint(1, self.N - 1))
        keys = [(key >> (8 * x) & 0xff) for x in range(0, 4)]
        return sum(self.salt[i] * (c+1)
                   for i, c in enumerate(keys)) % self.N
    
    header_template =  """
// --- ------------------------------------------- ---
// --- Following code is generated by perfect_hash ---
// --- Masks info (mask, elements):                ---
{mask_info}// --- ------------------------------------------- ---
const G_{BASENAME} : [u32;{size}] = $G;
"""
    hash_template= """
#[inline(always)]
fn hash_f_{basename}_{version}(key: u32) -> usize {{
    return (((key & 0xff )+1) * {s0} + ((key >> 8 & 0xff) + 1) * {s1} + ((key >> 16 & 0xff) + 1) * {s2} + ((key >> 24 )+ 1) * {s3}) as usize;
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

#[cfg(test)]
mod tests {{
use super::*;
const K_{BASENAME} : [u32;{size}] = {keys}
#[test]
pub fn test_{basename}() {{
    for i in 0..{size} {{
        assert!(find_{basename}(K_{BASENAME}[i]) == i)
    }}
}}
}}
"""

"""
File format:
First line: set_instruction_length_n
Rest: instruction_name,mask,match_bits,handler[,opcode_name]?
"""
def parse_key(filename):
    instructions = []
    try:
        with open(filename) as csvfile:
            reader = csv.reader(csvfile, delimiter=",")
            global SET_INSTRUCTION_LEN
            SET_INSTRUCTION_LEN = reader.__iter__().__next__()[0]
            for line in reader:
                opcode_name = None
                line = list(map(lambda x : str.strip(x), line))
                if len(line) == 5:
                    opcode_name = line[4]
                instructions.append(Instruction(line[0],line[1],line[2],line[3], opcode_name))
    except IOError:
        sys.exit("Error: Could not open {} for reading.".format(filename))
    return instructions

"""
Handle duplicate key by merging opcode and finding minimal mask
This only works if the opcodes that have the same keys 
have corrspond error checking in the handler. 
(Which must be the case, because by having the same handler, 
it must distinguish among different opcode in the handler)
"""
def handle_duplcaite_key(instructions):

    by_keys = dict()
    for inst in instructions:
        if inst.match_bits in by_keys:
            by_keys[inst.match_bits].append(inst)
        else:
            by_keys[inst.match_bits] = [inst]
    new_insts = []
    for (key, insts) in by_keys.items():
        if (len(insts) > 1):
            if False in [x.handler == insts[0].handler for x in insts]:
                sys.exit("Error: Duplicate key must have the same handler {}".format(
                    ))
            # produce minimal mask
            minimal_mask = reduce(lambda x, y: x & y, [int(x.mask, 16) for x in insts])
            insts[0].mask = hex(minimal_mask)
            new_insts.append(insts[0])
        else:
            new_insts.append(insts[0])
    print([str(x) for x in new_insts])
    return new_insts

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

    by_masks = get_by_mask(instructions)
    sorted_masks = [k for k in sorted(by_masks, key=lambda x: len(by_masks[x]), reverse=True)]
    mask_info = ""
    for mask in sorted_masks:
        mask_info += f"// ({mask}, {len(by_masks[mask])})\n"
    
    template = MyHash.header_template.format(mask_info=mask_info, BASENAME=str.upper(basename), size=len(G))
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
def build_hashmaps(basename, instructions):
    code = gen_hashmap_filter(basename, instructions)
    code += interface_template().format(basename=basename, size=len(instructions), SET_INSTRUCTION_LEN=SET_INSTRUCTION_LEN)
    return code

"""
Categorize elements by mask
"""
def get_by_mask(instructions):
    by_masks = dict()
    for inst in instructions:
        if inst.mask in by_masks:
            by_masks[inst.mask].append(inst)
        else:
            by_masks[inst.mask] = [inst]
    return by_masks


def main():
    usage = "usage: %prog KEYS_FILE"
    description = """\
Generate a hash-based decoder from perfect hash functions.
Program produce code in file [basename]_decoder.rs
Example input file see: TODO.
"""

    parser = OptionParser(usage = usage,
                          description = description,
                          prog = sys.argv[0],
                          version = "%prog: ")

    parser.add_option("-o", "--output",
                      action  = "store",
                      help    = "Specify output file explicitly."
                                "'-o std' to output to standard output",
                      metavar = "FILE")

    parser.add_option("--dir",
                      action  = "store",
                      help    = "Specify output directory.",
                      metavar = "FILE")
    
    options, args = parser.parse_args()
    if len(args) != 1:
        parser.error("Missing input file name")
    file = args[0]
    basename = os.path.basename(file).split('.')[0]

    if options.output:
        outname = options.output
    else:
        outname = basename+"_decoder.rs"

    insts = parse_key(file)
    insts = handle_duplcaite_key(insts)
    code = prelude()
    code += build_hashmaps(basename, insts)
    code += postlude(basename, insts)
    code += gen_test(basename, insts)

    if outname == 'std':
        stream = sys.stdout
    else:
        if options.dir:
            outname = os.path.join(options.dir, outname)
        try:
            stream = open(outname, 'w')
        except IOError:
            sys.exit("Error: Could not open {} for writing.".format(outname))
    stream.write(code)
    

if __name__ == '__main__':
    main()
