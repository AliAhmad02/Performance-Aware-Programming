use std::{fmt::Display, mem, ops::RangeInclusive};

const GENERIC_ARITHMETIC_OPCODES: [u8; 3] = [0, 0b001010, 0b001110];
const ACCUM_ARITHMETIC_OPCODES: [u8; 3] = [1, 0b001011, 0b001111];

const CJUMPS_OPCODE_RANGE1: RangeInclusive<u8> = 112..=127;
const CJUMPS_OPCODE_RANGE2: RangeInclusive<u8> = 224..=227;

pub fn simulate(binary: Vec<u8>) {
    let total_bits = binary.len();
    let mut registers: [u16; 8] = [0; 8];
    let mut flags: [bool; 2] = [false, false];
    let mut instruction_pointer = 0;
    let mut memory: [u8; u16::MAX as usize] = [0; u16::MAX as usize];

    while instruction_pointer < total_bits {
        let instruction = decode_instruction(&binary, instruction_pointer);
        instruction_pointer += instruction.size;
        match instruction.mnemonic {
            Mnemonic::MOV => simulate_move(
                instruction.src.unwrap(),
                instruction.dest,
                &mut registers,
                &mut memory,
            ),
            Mnemonic::ADD => {
                simulate_add(
                    instruction.src.unwrap(),
                    instruction.dest,
                    &mut registers,
                    &mut flags,
                );
            }
            Mnemonic::SUB => {
                simulate_sub(
                    instruction.src.unwrap(),
                    instruction.dest,
                    &mut registers,
                    &mut flags,
                );
            }
            Mnemonic::CMP => {
                simulate_cmp(
                    instruction.src.unwrap(),
                    instruction.dest,
                    &mut registers,
                    &mut flags,
                );
            }
            Mnemonic::JNE => {
                simulate_jne(instruction.dest, &flags, &mut instruction_pointer);
            }
            _ => unreachable!(),
        }
    }

    print_simulation_info(&registers, &flags, instruction_pointer, &memory);
}

fn print_simulation_info(
    registers: &[u16; 8],
    flags: &[bool; 2],
    i: usize,
    memory: &[u8; u16::MAX as usize],
) {
    let natural_order = [
        Register::AX as u8,
        Register::BX as u8,
        Register::CX as u8,
        Register::DX as u8,
        Register::SP as u8,
        Register::BP as u8,
        Register::SI as u8,
        Register::DI as u8,
    ];

    for idx in natural_order {
        println!("{}: {}", Register::from(idx), registers[(idx - 8) as usize],);
    }
    println!("ZF: {}, SF: {}", flags[0], flags[1]);
    println!("Instruction pointer: {i}");

    for (idx, value) in memory.iter().enumerate() {
        if value != &0 {
            println!("memory[{}] = {}", idx, value);
        }
    }
}

fn simulate_jne(destination: Operand, flags: &[bool; 2], i: &mut usize) {
    if !flags[0] {
        match destination {
            Operand::Immediate(value) => {
                *i = ((*i as u8) + value as u8) as usize;
            }
            _ => unreachable!(),
        }
    }
}

fn simulate_cmp(
    source: Operand,
    destination: Operand,
    registers: &mut [u16; 8],
    flags: &mut [bool; 2],
) {
    let sub =
        get_value_from_operand(destination, registers) - get_value_from_operand(source, registers);

    flags[0] = sub == 0;
    flags[1] = (sub & 0b1000000000000000) == 0b1000000000000000;
}

fn simulate_sub(
    source: Operand,
    destination: Operand,
    registers: &mut [u16; 8],
    flags: &mut [bool; 2],
) {
    let sub =
        get_value_from_operand(destination, registers) - get_value_from_operand(source, registers);

    flags[0] = sub == 0;
    flags[1] = (sub & 0b1000000000000000) == 0b1000000000000000;

    match destination {
        Operand::Register(register) => write_value_to_register(sub, register, registers),
        _ => unreachable!(),
    }
}

fn simulate_add(
    source: Operand,
    destination: Operand,
    registers: &mut [u16; 8],
    flags: &mut [bool; 2],
) {
    let sum =
        get_value_from_operand(destination, registers) + get_value_from_operand(source, registers);

    flags[0] = sum == 0;
    flags[1] = (sum & 0b1000000000000000) == 0b1000000000000000;

    match destination {
        Operand::Register(register) => write_value_to_register(sum, register, registers),
        _ => unreachable!(),
    }
}

fn simulate_move(
    source: Operand,
    destination: Operand,
    registers: &mut [u16; 8],
    memory: &mut [u8; u16::MAX as usize],
) {
    let value_src = get_value_from_operand(source, registers);
    let value_dest = get_value_from_operand(destination, registers);

    match (destination, source) {
        (Operand::Register(reg), Operand::Register(_))
        | (Operand::Register(reg), Operand::Immediate(_)) => {
            write_value_to_register(value_src, reg, registers)
        }
        (Operand::Register(reg), Operand::Memory(_, _, _)) => {
            let mem_value = if (reg as u8) > 8 {
                concatenate_bytes(memory[value_src as usize], memory[(value_src + 1) as usize])
            } else {
                memory[value_src as usize] as u16
            };
            write_value_to_register(mem_value, reg, registers);
        }
        (Operand::Memory(_, _, _), Operand::Register(reg)) => {
            if (reg as u8) > 8 {
                [
                    memory[(value_dest + 1) as usize],
                    memory[value_dest as usize],
                ] = value_src.to_be_bytes();
            } else {
                memory[value_dest as usize] = value_src as u8;
            }
        }
        // In this case, there is no way for me to know if the immediate
        // value is 8 or 16 bits. In assembly language, there is a
        // "word"/"byte" label to denote this. All the test cases just
        // have "word" on them, so we can assume it's always 16 bits
        (Operand::Memory(_, _, _), Operand::Immediate(_)) => {
            [
                memory[(value_dest + 1) as usize],
                memory[value_dest as usize],
            ] = value_src.to_be_bytes();
        }
        _ => unreachable!(),
    }
}

fn write_value_to_register(value: u16, register: Register, registers: &mut [u16; 8]) {
    let register = register as usize;
    if register < 4 {
        // Write to L registers
        registers[register] &= 0b1111111100000000;
        registers[register] |= value;
    } else if register < 8 {
        // Write to H registers
        registers[register - 4] &= 0b0000000011111111;
        registers[register - 4] |= value << 8;
    } else {
        // Write to 16 bit registers
        registers[register - 8] = value;
    }
}

fn get_value_from_operand(operand: Operand, registers: &[u16; 8]) -> u16 {
    match operand {
        Operand::Immediate(value) => value,
        Operand::Register(register) => get_value_from_register(register, registers),
        Operand::Memory(reg1, reg2, dis) => compute_memory_address(reg1, reg2, dis, registers),
    }
}

fn get_value_from_register(register: Register, registers: &[u16; 8]) -> u16 {
    let register = register as usize;

    if register < 4 {
        // Get low bits
        (registers[register] as u8) as u16
    } else if register < 8 {
        // Get high bits
        registers[register - 4] >> 8
    } else {
        // Get full register
        registers[register - 8]
    }
}

fn compute_memory_address(
    register1: Option<Register>,
    register2: Option<Register>,
    displacement: Option<u16>,
    registers: &[u16; 8],
) -> u16 {
    let mut idx = 0;
    if let Some(reg) = register1 {
        idx += get_value_from_register(reg, registers);
    }
    if let Some(reg) = register2 {
        idx += get_value_from_register(reg, registers);
    }
    if let Some(dis) = displacement {
        idx += dis;
    }

    idx
}

pub fn decode_binary_and_print(binary: Vec<u8>) {
    let mut i = 0;
    println!("bits 16");

    while i < binary.len() {
        let instruction = decode_instruction(&binary, i);

        i += instruction.size;

        println!("{}", instruction);
    }
}

fn decode_instruction(binary: &[u8], i: usize) -> Instruction {
    let info = get_standard_bits_from_word(binary[i], binary[i + 1]);
    if (info.opcode >> 2) == 0b1011 {
        decode_immediate_to_register_mov(binary, i)
    } else if (info.opcode == 0b110001) && info.d {
        decode_immediate_to_memory_mov(binary, i, &info)
    } else if info.opcode == 0b100010 {
        decode_generic_mov(binary, i, &info)
    } else if GENERIC_ARITHMETIC_OPCODES.contains(&info.opcode) {
        decode_generic_arithmetic(binary, i, &info)
    } else if info.opcode == 0b100000 {
        decode_immediate_arithmetic(binary, i, &info)
    } else if ACCUM_ARITHMETIC_OPCODES.contains(&info.opcode) && !info.d {
        decode_accum_arithmetic(binary, i, &info)
    } else if CJUMPS_OPCODE_RANGE1.contains(&binary[i]) || CJUMPS_OPCODE_RANGE2.contains(&binary[i])
    {
        decode_cjumps(binary, i)
    } else {
        unreachable!()
    }
}

fn decode_cjumps(binary: &[u8], i: usize) -> Instruction {
    Instruction {
        mnemonic: Mnemonic::from(binary[i]),
        dest: Operand::Immediate(binary[i + 1] as u16),
        src: None,
        size: 2,
    }
}

fn decode_accum_arithmetic(binary: &[u8], i: usize, info: &InstructionInfo) -> Instruction {
    let mnemonic = Mnemonic::from((info.opcode >> 1) & 0b00000111);
    if info.w {
        Instruction {
            mnemonic,
            dest: Operand::Register(Register::AX),
            src: Some(Operand::Immediate(concatenate_bytes(
                binary[i + 1],
                binary[i + 2],
            ))),
            size: 3,
        }
    } else {
        Instruction {
            mnemonic,
            dest: Operand::Register(Register::AL),
            src: Some(Operand::Immediate(binary[i + 1] as u16)),
            size: 2,
        }
    }
}

fn decode_immediate_arithmetic(binary: &[u8], i: usize, info: &InstructionInfo) -> Instruction {
    let mnemonic = Mnemonic::from(info.reg);
    let (destination, increment) = decode_rm_value_generic(binary, i, info);

    if info.w && !info.d {
        Instruction {
            mnemonic,
            dest: destination,
            src: Some(Operand::Immediate(concatenate_bytes(
                binary[i + increment],
                binary[i + increment + 1],
            ))),
            size: increment + 2,
        }
    } else {
        Instruction {
            mnemonic,
            dest: destination,
            src: Some(Operand::Immediate(binary[i + increment] as u16)),
            size: increment + 1,
        }
    }
}

fn decode_generic_arithmetic(binary: &[u8], i: usize, info: &InstructionInfo) -> Instruction {
    let mnemonic = Mnemonic::from((info.opcode >> 1) & 0b00000111);
    let (destination, source, increment) = decode_generic(binary, i, info);
    Instruction {
        mnemonic,
        dest: destination,
        src: Some(source),
        size: increment,
    }
}

fn decode_generic_mov(binary: &[u8], i: usize, info: &InstructionInfo) -> Instruction {
    let (destination, source, increment) = decode_generic(binary, i, info);
    Instruction {
        mnemonic: Mnemonic::MOV,
        dest: destination,
        src: Some(source),
        size: increment,
    }
}

fn decode_generic(binary: &[u8], i: usize, info: &InstructionInfo) -> (Operand, Operand, usize) {
    let mut destination = Operand::Register(Register::from(info.reg + 8 * (info.w as u8)));

    let (mut source, increment) = decode_rm_value_generic(binary, i, info);
    if !info.d {
        mem::swap(&mut destination, &mut source);
    }

    (destination, source, increment)
}

fn decode_rm_value_generic(binary: &[u8], i: usize, info: &InstructionInfo) -> (Operand, usize) {
    match info.mod_ {
        0b11 => (
            Operand::Register(Register::from(info.rm + 8 * (info.w as u8))),
            2,
        ),
        0b00 if info.rm == 0b110 => (
            Operand::build_memory(8, Some(concatenate_bytes(binary[i + 2], binary[i + 3]))),
            4,
        ),
        0b00 => (Operand::build_memory(info.rm, None), 2),
        0b01 => (
            Operand::build_memory(info.rm, Some(binary[i + 2] as u16)),
            3,
        ),
        0b10 => (
            Operand::build_memory(
                info.rm,
                Some(concatenate_bytes(binary[i + 2], binary[i + 3])),
            ),
            4,
        ),
        _ => unreachable!(),
    }
}

// It's immediate to register/memory even though I've just called it to memory.
// Don't understand why it needs to also do immediate to register since we have
// the one right below, but the manual has both
fn decode_immediate_to_memory_mov(binary: &[u8], i: usize, info: &InstructionInfo) -> Instruction {
    let (destination, increment) = decode_rm_value_generic(binary, i, info);

    if info.w {
        Instruction {
            mnemonic: Mnemonic::MOV,
            dest: destination,
            src: Some(Operand::Immediate(concatenate_bytes(
                binary[i + increment],
                binary[i + increment + 1],
            ))),
            size: increment + 2,
        }
    } else {
        Instruction {
            mnemonic: Mnemonic::MOV,
            dest: destination,
            src: Some(Operand::Immediate(binary[i + increment] as u16)),
            size: increment + 1,
        }
    }
}
fn decode_immediate_to_register_mov(binary: &[u8], i: usize) -> Instruction {
    let w = (0b00001000 & binary[i]) == 0b00001000;
    let register = Register::from((0b00000111 & binary[i]) + (w as u8) * 8);
    let (immediate, increment) = if w {
        (concatenate_bytes(binary[i + 1], binary[i + 2]), 3)
    } else {
        (binary[i + 1] as u16, 2)
    };
    Instruction {
        mnemonic: Mnemonic::MOV,
        dest: Operand::Register(register),
        src: Some(Operand::Immediate(immediate)),
        size: increment,
    }
}

fn concatenate_bytes(left: u8, right: u8) -> u16 {
    // The 16 bit value is built up by concatenating
    // the left bytes onto the end of the right bytes.
    // I think this is because it's little endian ?
    (right as u16) << 8 | (left as u16)
}

fn get_standard_bits_from_word(left: u8, right: u8) -> InstructionInfo {
    InstructionInfo {
        opcode: left >> 2,
        d: (0b00000010 & left) == 0b00000010,
        w: (0b00000001 & left) == 0b00000001,
        mod_: right >> 6,
        reg: (right >> 3) & 0b00000111,
        rm: right & 0b00000111,
    }
}

struct InstructionInfo {
    // This field really just refers to the first 6
    // bits. Often, that is indeed the opcode, but
    // it is not always the case.
    opcode: u8,
    // Note: d here sometimes also means s
    d: bool,
    w: bool,
    mod_: u8,
    reg: u8,
    rm: u8,
}

pub struct Instruction {
    mnemonic: Mnemonic,
    dest: Operand,
    src: Option<Operand>,
    size: usize,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.src {
            Some(src) => write!(f, "{} {}, {}", self.mnemonic, self.dest, src),
            None => write!(f, "{} {}", self.mnemonic, self.dest),
        }
    }
}

enum Mnemonic {
    MOV,
    ADD,
    SUB,
    CMP,
    JO,
    JNO,
    JB,
    JAE,
    JE,
    JNE,
    JBE,
    JA,
    JS,
    JNS,
    JP,
    JNP,
    JL,
    JGE,
    JLE,
    JG,
    LOOPNZ,
    LOOPZ,
    LOOP,
    JCXZ,
}

impl From<u8> for Mnemonic {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::MOV,
            0 => Self::ADD,
            5 => Self::SUB,
            7 => Self::CMP,
            112 => Self::JO,
            113 => Self::JNO,
            114 => Self::JB,
            115 => Self::JAE,
            116 => Self::JE,
            117 => Self::JNE,
            118 => Self::JBE,
            119 => Self::JA,
            120 => Self::JS,
            121 => Self::JNS,
            122 => Self::JP,
            123 => Self::JNP,
            124 => Self::JL,
            125 => Self::JGE,
            126 => Self::JLE,
            127 => Self::JG,
            224 => Self::LOOPNZ,
            225 => Self::LOOPZ,
            226 => Self::LOOP,
            227 => Self::JCXZ,
            _ => unreachable!(),
        }
    }
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MOV => "mov",
                Self::ADD => "add",
                Self::SUB => "sub",
                Self::CMP => "cmp",
                Self::JO => "jo",
                Self::JNO => "jno",
                Self::JB => "jb",
                Self::JAE => "jae",
                Self::JE => "je",
                Self::JNE => "jne",
                Self::JBE => "jbe",
                Self::JA => "ja",
                Self::JS => "js",
                Self::JNS => "jns",
                Self::JP => "jp",
                Self::JNP => "jnp",
                Self::JL => "jl",
                Self::JGE => "jge",
                Self::JLE => "jle",
                Self::JG => "jg",
                Self::LOOPNZ => "loopnz",
                Self::LOOPZ => "loopz",
                Self::LOOP => "loop",
                Self::JCXZ => "jcxz",
            }
        )
    }
}

#[derive(Copy, Clone)]
enum Operand {
    Immediate(u16),
    Memory(Option<Register>, Option<Register>, Option<u16>),
    Register(Register),
}

impl Operand {
    fn build_memory(rm: u8, displacement: Option<u16>) -> Operand {
        let (register1, register2) = match rm {
            0 => (Some(Register::BX), Some(Register::SI)),
            1 => (Some(Register::BX), Some(Register::DI)),
            2 => (Some(Register::BP), Some(Register::SI)),
            3 => (Some(Register::BP), Some(Register::DI)),
            4 => (Some(Register::SI), None),
            5 => (Some(Register::DI), None),
            6 => (Some(Register::BP), None),
            7 => (Some(Register::BX), None),
            8 => (None, None),
            _ => unreachable!(),
        };
        Self::Memory(register1, register2, displacement)
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate(value) => write!(f, "{}", value),
            Self::Register(register) => write!(f, "{}", register),
            Self::Memory(Some(register1), Some(register2), Some(value)) => {
                write!(f, "[{} + {} + {}]", register1, register2, value)
            }
            Self::Memory(Some(register1), Some(register2), None) => {
                write!(f, "[{} + {}]", register1, register2)
            }
            Self::Memory(Some(register), None, Some(value))
            | Self::Memory(None, Some(register), Some(value)) => {
                write!(f, "[{} + {}]", register, value)
            }
            Self::Memory(Some(register), None, None) | Self::Memory(None, Some(register), None) => {
                write!(f, "[{}]", register)
            }
            Self::Memory(None, None, Some(value)) => write!(f, "[{}]", value),
            Self::Memory(None, None, None) => unreachable!(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum Register {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::AL,
            1 => Self::CL,
            2 => Self::DL,
            3 => Self::BL,
            4 => Self::AH,
            5 => Self::CH,
            6 => Self::DH,
            7 => Self::BH,
            8 => Self::AX,
            9 => Self::CX,
            10 => Self::DX,
            11 => Self::BX,
            12 => Self::SP,
            13 => Self::BP,
            14 => Self::SI,
            15 => Self::DI,
            _ => unreachable!(),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AL => "al",
                Self::CL => "cl",
                Self::DL => "dl",
                Self::BL => "bl",
                Self::AH => "ah",
                Self::CH => "ch",
                Self::DH => "dh",
                Self::BH => "bh",
                Self::AX => "ax",
                Self::CX => "cx",
                Self::DX => "dx",
                Self::BX => "bx",
                Self::SP => "sp",
                Self::BP => "bp",
                Self::SI => "si",
                Self::DI => "di",
            }
        )
    }
}
