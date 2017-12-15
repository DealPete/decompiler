use defs::*;
use std::fmt;
use std::ops::Add;
use std::ops::Sub;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone)]
pub struct State<'program> {
    pub cs : u16,
    pub ip : u16,
    regs : Registers,
    flags : Flags,
    pub load_module : &'program LoadModule,
    memory : HashMap<usize, Byte>
}

impl<'program> State<'program> {
    pub fn new(load_module: &'program LoadModule) -> State<'program> {
        State {
            cs : 0,
            ip : 0,
            regs : Registers::new(),
            flags : Flags::new(),
            load_module: load_module,
            memory : HashMap::new()
        }
    }
    
    pub fn next_inst_address(&self) -> usize {
        16 * self.cs as usize + self.ip as usize
    }

    pub fn union(self, state: State<'program>) -> State<'program> {
        if self.cs != state.cs || self.ip != state.ip {
            panic!("Unifying states with different cs/ip unimplemented");
        };
        State {
            cs: self.cs,
            ip: self.ip,
            regs: self.regs.union(state.regs),
            flags: self.flags.union(state.flags),
            load_module: state.load_module,
            memory: {
                let mut new_memory = HashMap::new();
                for (offset, byte) in self.memory {
                    new_memory.insert(offset, byte);
                }
                for (offset, byte) in state.memory {
                    new_memory.insert(offset, byte);
                }
                new_memory
            }
        }
    }

    pub fn is_subset(&self, state: &State) -> bool {
        for (offset, byte1) in self.memory.iter() {
            match state.memory.get(&offset) {
                None => return false,
                Some(ref byte2) => if !byte1.is_subset(byte2) {
                    return false;
                }
            }
        };
        self.cs == state.cs &&
        self.ip == state.ip &&
        self.regs.is_subset(&state.regs) &&
        self.flags.is_subset(state.flags)
    }

    pub fn get_reg8(&self, reg: Register) -> Byte {
        match reg {
            Register::AL => self.regs.ax.split_low(),
            Register::AH => self.regs.ax.split_high(),
            Register::BL => self.regs.bx.split_low(),
            Register::BH => self.regs.bx.split_high(),
            Register::CL => self.regs.cx.split_low(),
            Register::CH => self.regs.cx.split_high(),
            Register::DL => self.regs.dx.split_low(),
            Register::DH => self.regs.dx.split_high(),
            _ => panic!("register {:?} is not a byte register.", reg)
        }
    }

    pub fn get_reg16(&self, reg: Register) -> Word {
        match reg {
            Register::AX => self.regs.ax.clone(),
            Register::BX => self.regs.bx.clone(),
            Register::CX => self.regs.cx.clone(),
            Register::DX => self.regs.dx.clone(),
            Register::CS => {
                let mut set = HashSet::new();
                set.insert(self.cs);
                Word::Int(set)
            },
            Register::DS => self.regs.ds.clone(),
            Register::ES => self.regs.es.clone(),
            Register::SS => self.regs.ss.clone(),
            Register::BP => self.regs.bp.clone(),
            Register::SP => self.regs.sp.clone(),
            _ => panic!("register {:?} is not a word register.", reg)
        }
    }

    pub fn set_reg8(self, reg: Register, value: Byte) -> State<'program> {
        State {
            regs: match reg {
                Register::AL => Registers {
                    ax: Word::Bytes(value, self.get_reg8(Register::AH)),
                    .. self.regs
                },
                Register::AH => Registers {
                    ax: Word::Bytes(self.get_reg8(Register::AL), value),
                    .. self.regs
                },
                Register::BL => Registers {
                    bx: Word::Bytes(value, self.get_reg8(Register::BH)),
                    .. self.regs
                },
                Register::BH => Registers {
                    bx: Word::Bytes(self.get_reg8(Register::BL), value),
                    .. self.regs
                },
                Register::CL => Registers {
                    cx: Word::Bytes(value, self.get_reg8(Register::CH)),
                    .. self.regs
                },
                Register::CH => Registers {
                    cx: Word::Bytes(self.get_reg8(Register::CL), value),
                    .. self.regs
                },
                Register::DL => Registers {
                    dx: Word::Bytes(value, self.get_reg8(Register::DH)),
                    .. self.regs
                },
                Register::DH => Registers {
                    dx: Word::Bytes(self.get_reg8(Register::DL), value),
                    .. self.regs
                },
            _ => panic!("can't set byte in register {:?}.", reg)
            },
            .. self
        }
    }

    pub fn set_reg16(self, reg: Register, value: Word) -> State<'program> {
        State {
            regs: match reg {
                Register::AX => Registers { ax: value, .. self.regs },
                Register::BX => Registers { bx: value, .. self.regs },
                Register::CX => Registers { cx: value, .. self.regs },
                Register::DX => Registers { dx: value, .. self.regs },
                Register::CS => panic!("You can't set CS directly on x86."),
                Register::SP => Registers { sp: value, .. self.regs },
                Register::SS => Registers { ss: value, .. self.regs },
                Register::BP => Registers { bp: value, .. self.regs },
                Register::SI => Registers { si: value, .. self.regs },
                Register::DI => Registers { di: value, .. self.regs },
                Register::DS => Registers { ds: value, .. self.regs },
                Register::ES => Registers { es: value, .. self.regs },
            _ => panic!("can't set word in register {:?}.", reg)
            },
            .. self
        }
    }

    pub fn get_flags(&self) -> Flags {
        self.flags
    }

    pub fn set_flags(self, flags: Flags) -> State<'program> {
        State {
            flags: flags,
            .. self
        }
    }

    pub fn get_value(&self, operand: Operand) -> Value {
        match operand {
            Operand::Register8(_) | Operand::Imm8(_) =>
                Value::Byte(self.get_byte(operand)),
            _ => Value::Word(self.get_word(operand))
        }
    }

    pub fn get_byte(&self, operand: Operand) -> Byte {
        match operand {
            Operand::Register16(_) | Operand::Imm16(_) =>
                panic!("can't get word from byte source"),
            Operand::Register8(reg) => self.get_reg8(reg),
            Operand::Imm8(imm) => Byte::new(imm as u8),
            Operand::SegPtr(segment, pointer) => match self.get_reg16(segment) {
                Word::Undefined => Byte::Undefined,
                Word::AnyValue => panic!("trying to read from unlimited segment"),
                Word::Int(segments) => {
                    match self.pointer_offset(pointer) {
                        Word::Undefined => Byte::Undefined,
                        Word::AnyValue => panic!("trying to read from unlimited offset"),
                        Word::Int(offsets) => self.read_memory_byte(segments, offsets),
                        _ => panic!("shouldn't be here")
                    }
                },
                _ => panic!("Invalid value for segment.")
            }
        }
    }

    pub fn get_word(&self, operand: Operand) -> Word {
        match operand {
            Operand::Register8(_) | Operand::Imm8(_) =>
                panic!("can't get word from byte source"),
            Operand::Register16(reg) => self.get_reg16(reg),
            Operand::Imm16(imm) => Word::new(imm as u16),
            Operand::SegPtr(segment, pointer) => match self.get_reg16(segment) {
                Word::Undefined => Word::Undefined,
                Word::AnyValue => panic!("trying to read from unlimited segment"),
                Word::Int(segments) => {
                    match self.pointer_offset(pointer) {
                        Word::Undefined => Word::Undefined,
                        Word::AnyValue => panic!("trying to read from unlimited offset"),
                        Word::Int(offsets) => self.read_memory_word(segments, offsets),
                        _ => panic!("shouldn't be here")
                    }
                },
                _ => panic!("Invalid value for segment.")
            }
        }
    }

    pub fn get_combined_word(&self, operand: Operand) -> Word {
        let word = self.get_word(operand);
        if let Word::Bytes(bytel, byteh) = word {
            bytel.combine(byteh)
        } else {
            word
        }
    }

    pub fn set_value(self, operand: Operand, value: Value) -> State<'program> {
        match value {
            Value::Word(word) => self.set_word(operand, word),
            Value::Byte(byte) => self.set_byte(operand, byte)
        }
    }

    pub fn set_word(self, operand: Operand, word: Word) -> State<'program> {
        match operand {
            Operand::Register16(target_reg) =>
                self.set_reg16(target_reg, word),
            Operand::SegPtr(segment, pointer) => match self.get_reg16(segment) {
                Word::Undefined => self,
                Word::AnyValue =>
                    panic!("trying to write to memory with unlimited segment."),
                Word::Int(segments) => {
                    match self.pointer_offset(pointer) {
                        Word::Undefined =>
                            panic!("trying to write to undefined memory offset."),
                        Word::AnyValue =>
                            panic!("trying to write to unlimited memory offset."),
                        Word::Int(offsets) => {
                            self.write_memory(segments, offsets, Value::Word(word))
                        },
                        _ => panic!("shouldn't be here!")
                    }
                },
                _ => panic!("invalid segment value.")
            },
            _ => panic!("Unimplemented target operand for set_word")
        }
    }

    pub fn clear_value(self, operand: Operand) -> State<'program> {
        match self.get_value(operand) {
            Value::Word(_) => self.set_word(operand, Word::new(0)),
            Value::Byte(_) => self.set_byte(operand, Byte::new(0))
        }
    }

    pub fn set_byte(self, operand: Operand, byte: Byte) -> State<'program> {
        match operand {
            Operand::Register8(target_reg) =>
                self.set_reg8(target_reg, byte),
            Operand::SegPtr(segment, pointer) => match self.get_reg16(segment) {
                Word::Undefined => self,
                Word::AnyValue =>
                    panic!("trying to write to memory with unlimited segment."),
                Word::Int(segments) => {
                    match self.pointer_offset(pointer) {
                        Word::Undefined =>
                            panic!("trying to write to undefined memory offset."),
                        Word::AnyValue =>
                            panic!("trying to write to unlimited memory offset."),
                        Word::Int(offsets) =>
                            self.write_memory(segments, offsets, Value::Byte(byte)),
                        _ => panic!("shouldn't be here!")
                    }
                },
                _ => panic!("invalid segment value.")
            },
            _ => panic!("Unimplemented target for set_byte_op")
        }
    }

    fn read_memory_byte(&self, segments: HashSet<u16>, offsets: HashSet<u16>) -> Byte {
        let mut byte = Byte::Int(HashSet::new());
        for segment in segments {
            for offset in offsets.iter() {
                let location = 16*(segment as usize) + *offset as usize;
                byte = byte.union(match self.memory.get(&location) {
                    Some(new_byte) => new_byte.clone(),
                    None => Byte::new(self.load_module.buffer[
                        location - 16*(self.load_module.memory_segment as usize)
                    ])
                });
            }
        }
        byte
    }

    fn read_memory_word(&self, segments: HashSet<u16>, offsets: HashSet<u16>) -> Word {
        self.read_memory_byte(segments.clone(), offsets.clone()).combine(
            self.read_memory_byte(segments,
                offsets.iter().map(|offset| {offset + 1}).collect()))
    }

    fn write_memory(self, segments: HashSet<u16>, offsets: HashSet<u16>, value: Value) -> State<'program> {
        let mut new_memory = self.memory.clone();
        for segment in segments {
            for offset in offsets.iter() {
                let location = 16*(segment as usize) + *offset as usize;
                match value {
                    Value::Word(ref word) => {
                        new_memory.insert(location, word.clone().split_low());
                        new_memory.insert(location + 1, word.clone().split_high());
                    },
                    Value::Byte(ref byte) => {
                        new_memory.insert(location, byte.clone());
                    }
                }
            }
        };
        State {
            memory: new_memory,
            .. self
        }
    }

    fn pointer_offset(&self, pointer: Pointer) -> Word {
        match pointer {
            Pointer::Disp16(offset) => Word::new(offset),
            Pointer::Reg(register) => self.get_reg16(register),
            Pointer::RegReg(register1, register2) =>
                self.get_reg16(register1) + self.get_reg16(register2),
            Pointer::RegDisp8(register, byte) =>
                self.get_reg16(register) + Word::new(byte as u16),
            Pointer::RegRegDisp8(register1, register2, byte) =>
                self.get_reg16(register1) 
                + self.get_reg16(register2)
                + Word::new(byte as u16),
            Pointer::RegDisp16(register, offset) =>
                self.get_reg16(register) + Word::new(offset),
            Pointer::RegRegDisp16(register1, register2, offset) =>
                self.get_reg16(register1)
                + self.get_reg16(register2)
                + Word::new(offset)
        }
    }
}

impl<'program> fmt::Display for State<'program> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let regs = &self.regs;
        let line1 = format!("AX={}  BX={}  CX={}  DX={}  SP={}  BP={}  SI={}  DI={}",
        regs.ax, regs.bx, regs.cx, regs.dx, regs.sp, regs.bp, regs.si, regs.di);
        let line2 = format!("DS={}  ES={}  SS={}  CS={:04x}  IP={:04x}",
            regs.ds, regs.es, regs.ss, self.cs, self.ip);
        let mut memory = String::new();
        for (address, value) in self.memory.iter() {
            memory.push_str(format!("[{:x}] = {}\t", address, value).as_str());
        };
        return write!(f, "{}\n{}\n{}", line1, line2, memory);
    }
}

#[derive(Clone)]
pub struct Registers {
    ax : Word,
    bx : Word,
    cx : Word,
    dx : Word,
    sp : Word,
    bp : Word,
    si : Word,
    di : Word,
    ds : Word,
    es : Word,
    ss : Word
}

impl Registers {
    pub fn new() -> Registers {
        return Registers {
            ax : Word::Undefined,
            bx : Word::Undefined,
            cx : Word::Undefined,
            dx : Word::Undefined,
            sp : Word::Undefined,
            bp : Word::Undefined,
            si : Word::Undefined,
            di : Word::Undefined,
            ds : Word::Undefined,
            es : Word::Undefined,
            ss : Word::Undefined
        }
    }

    pub fn union(self, regs: Registers) -> Registers {
        Registers {
            ax: self.ax.union(regs.ax),
            bx: self.bx.union(regs.bx),
            cx: self.cx.union(regs.cx),
            dx: self.dx.union(regs.dx),
            sp: self.sp.union(regs.sp),
            bp: self.bp.union(regs.bp),
            si: self.si.union(regs.si),
            di: self.di.union(regs.di),
            ds: self.ds.union(regs.ds),
            es: self.es.union(regs.es),
            ss: self.ss.union(regs.ss)
        }
    }

    pub fn is_subset(&self, regs: &Registers) -> bool {
        self.ax.is_subset(&regs.ax) &&
        self.bx.is_subset(&regs.bx) &&
        self.cx.is_subset(&regs.cx) &&
        self.dx.is_subset(&regs.dx) &&
        self.sp.is_subset(&regs.sp) &&
        self.bp.is_subset(&regs.bp) &&
        self.si.is_subset(&regs.si) &&
        self.di.is_subset(&regs.di) &&
        self.ds.is_subset(&regs.ds) &&
        self.es.is_subset(&regs.es) &&
        self.ss.is_subset(&regs.ss)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Flags {
    pub carry : Flag,
    pub parity : Flag,
    pub adjust : Flag,
    pub zero : Flag,
    pub sign : Flag,
    pub int : Flag,
    pub dir : Flag,
    pub overflow : Flag
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            carry: Flag::Undefined,
            parity: Flag::Undefined,
            adjust: Flag::Undefined,
            zero: Flag::Undefined,
            sign: Flag::Undefined,
            int: Flag::Undefined,
            dir: Flag::Undefined,
            overflow: Flag::Undefined,
        }
    }

    pub fn union(self, flags: Flags) -> Flags {
        Flags {
            carry: self.carry.union(flags.carry),
            parity: self.parity.union(flags.parity),
            adjust: self.adjust.union(flags.adjust),
            zero: self.zero.union(flags.zero),
            sign: self.sign.union(flags.sign),
            int: self.int.union(flags.int),
            dir: self.dir.union(flags.dir),
            overflow: self.overflow.union(flags.overflow),
        }
    }

    fn is_subset(&self, flags: Flags) -> bool {
        self.carry.is_subset(flags.carry) &&
        self.parity.is_subset(flags.parity) &&
        self.adjust.is_subset(flags.adjust) &&
        self.zero.is_subset(flags.zero) &&
        self.sign.is_subset(flags.sign) &&
        self.int.is_subset(flags.int) &&
        self.dir.is_subset(flags.dir) &&
        self.overflow.is_subset(flags.overflow)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Flag {
    Undefined,
    True,
    False,
    TrueAndFalse
}

impl Flag {
    pub fn union(self, flag: Flag) -> Flag {
        match self {
            Flag::Undefined => flag,
            Flag::True => match flag {
                Flag::Undefined => Flag::True,
                Flag::False => Flag::TrueAndFalse,
                _ => flag
            },
            Flag::False => match flag {
                Flag::Undefined => Flag::False,
                Flag::True => Flag::TrueAndFalse,
                _ => flag
            },
            Flag::TrueAndFalse => Flag::TrueAndFalse
        }
    }

    fn is_subset(&self, flag: Flag) -> bool {
        match *self {
            Flag::Undefined => match flag {
                Flag::Undefined | Flag::TrueAndFalse => true,
                _ => false
            },
            Flag::TrueAndFalse => flag == Flag::TrueAndFalse,
            _ => *self == flag
        }
    }

    pub fn add_true(&mut self) {
        *self = match *self {
            Flag::Undefined | Flag::True =>
                Flag::True,
            Flag::False | Flag::TrueAndFalse
                => Flag::TrueAndFalse
        }
    }

    pub fn add_false(&mut self) {
        *self = match *self {
            Flag::Undefined | Flag::False =>
                Flag::False,
            Flag::True | Flag::TrueAndFalse
                => Flag::TrueAndFalse
        }
    }
}
#[derive(Clone, PartialEq)]
pub enum Value {
    Word(Word),
    Byte(Byte)
}

#[derive(Clone, PartialEq)]
pub enum Word {
    Undefined,
    AnyValue,
    Int(HashSet<u16>),
    Bytes(Byte, Byte),
}

impl Word {
    pub fn new(word: u16) -> Word {
        let mut word_set = HashSet::new();
        word_set.insert(word);
        Word::Int(word_set)
    }

    fn union(self, word: Word) -> Word {
        if let Word::Bytes(byte1, byte2) = word {
            Word::Bytes(byte1.union(self.split_low()),
                byte2.union(self.split_high()))
        } else if let Word::Bytes(byte1, byte2) = self { 
            Word::Bytes(byte1.union(word.split_low()),
                byte2.union(word.split_high()))
        } else {
            match self {
                Word::Undefined => Word::Undefined,
                Word::AnyValue => Word::AnyValue,
                Word::Int(set1) => match word {
                    Word::Undefined => Word::Undefined,
                    Word::AnyValue => Word::AnyValue,
                    Word::Int(set2) => Word::Int(set1.union(&set2).cloned().collect()),
                    _ => panic!("invalid word")
                    },
                _ => panic!("invalid word")
            }
        }
    }

    fn is_subset(&self, word: &Word) -> bool {
        if let Word::Bytes(ref byte1, ref byte2) = *word {
            self.split_low().is_subset(byte1) &&
            self.split_high().is_subset(byte2)
        } else if let Word::Bytes(ref byte1, ref byte2) = *self {
            byte1.is_subset(&word.split_low()) &&
            byte2.is_subset(&word.split_high())
        } else {
            match *self {
                Word::Undefined => match *word {
                    Word::Undefined | Word::AnyValue => true,
                    _ => false
                },
                Word::AnyValue => match *word {
                    Word::AnyValue => true,
                    _ => false
                },
                Word::Int(ref set1) => match *word {
                    Word::Undefined => false,
                    Word::AnyValue => true,
                    Word::Int(ref set2) => set1.is_subset(&set2),
                    _ => panic!("shouldn't be here")
                },
                _ => panic!("shouldn't be here")
            }
        }
    }

    fn split_low(&self) -> Byte {
        match *self {
            Word::Undefined => Byte::Undefined,
            Word::AnyValue => Byte::AnyValue,
            Word::Int(ref words) =>
                Byte::Int({
                    let mut set = HashSet::new();
                    for word in words.iter() {
                        set.insert(*word as u8);
                    };
                    set
                }),
            Word::Bytes(ref byte_low, _) => byte_low.clone()
        }
    }

    fn split_high(&self) -> Byte {
        match *self {
            Word::Undefined => Byte::Undefined,
            Word::AnyValue => Byte::AnyValue,
            Word::Int(ref words) =>
                Byte::Int({
                    let mut set = HashSet::new();
                    for word in words {
                        set.insert((*word >> 8) as u8);
                    };
                    set
                }),
            Word::Bytes(_, ref byte_high) => byte_high.clone()
        }
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Word::Undefined => String::from("????"),
            Word::AnyValue => String::from("****"),
            Word::Int(ref set) =>
                if set.len() == 1 {
                    format!("{:04x}", set.iter().collect::<Vec<&u16>>()[0])
                } else {
                    String::from("{--}")
                },
            Word::Bytes(ref reg1, ref reg2) => format!("{}{}", reg2, reg1)
        })
    }
}

impl Add<Word> for Word {
    type Output = Word;
    fn add(self, rhs: Word) -> Word {
        let word1 = if let Word::Bytes(bytel, byteh) = self {
            bytel.combine(byteh)
        } else {
            self.clone()
        };
        let word2 = if let Word::Bytes(bytel, byteh) = rhs {
            bytel.combine(byteh)
        } else {
            rhs.clone()
        };
        match word1 {
            Word::Undefined => Word::Undefined,
            Word::AnyValue =>
                if word2 == Word::Undefined {
                    Word::Undefined
                } else {
                    Word::AnyValue
                },
            Word::Int(set1) => match word2 {
                Word::Undefined => Word::Undefined,
                Word::AnyValue => Word::AnyValue,
                Word::Int(set2) => {
                    let mut set = HashSet::new();
                    for word1 in set1 {
                        for word2 in set2.clone() {
                            set.insert(word1 + word2);
                        }
                    };
                    Word::Int(set)
                },
                _ => panic!("shouldn't be here")
            },
            _ => panic!("shouldn't be here")
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Byte {
    Undefined,
    AnyValue,
    Int(HashSet<u8>),
}

impl Byte {
    pub fn new(byte: u8) -> Byte {
        let mut byte_set = HashSet::new();
        byte_set.insert(byte);
        Byte::Int(byte_set)
    }

    pub fn to_word(self) -> Word {
        match self {
            Byte::Undefined => Word::Undefined,
            Byte::AnyValue => Word::AnyValue,
            Byte::Int(set) => {
                let mut words = HashSet::new();
                for byte in set {
                    words.insert(byte as u16);
                }
                Word::Int(words)
            }
        }
    }

    pub fn combine(self, byte: Byte) -> Word {
        match self {
            Byte::Undefined => Word::Undefined,
            Byte::AnyValue => match byte {
                Byte::Undefined => Word::Undefined,
                Byte::AnyValue => Word::AnyValue,
                Byte::Int(seth) => {
                    let mut words = HashSet::new();
                    for bytel in 0..256 {
                        for byteh in seth.iter() {
                            words.insert(bytel + ((*byteh as u16) << 8));
                        }
                    }
                    Word::Int(words)
                }
            },
            Byte::Int(setl) => match byte {
                Byte::Undefined => Word::Undefined,
                Byte::AnyValue => Word::AnyValue,
                Byte::Int(seth) => {
                    let mut words = HashSet::new();
                    for bytel in setl {
                        for byteh in seth.iter() {
                            words.insert(bytel as u16 + ((*byteh as u16) << 8));
                        }
                    }
                    Word::Int(words)
                }
            }
        }
    }

    fn union(self, byte: Byte) -> Byte {
        match self {
            Byte::Undefined => Byte::Undefined,
            Byte::AnyValue => Byte::AnyValue,
            Byte::Int(set1) => match byte {
                Byte::Undefined => Byte::Undefined,
                Byte::AnyValue => Byte::AnyValue,
                Byte::Int(set2) => Byte::Int(set1.union(&set2).cloned().collect())
            }
        }
    }


    fn is_subset(&self, byte: &Byte) -> bool {
        match *self {
            Byte::Undefined => match *byte {
                Byte::Undefined | Byte::AnyValue => true,
                _ => false
            },
            Byte::AnyValue => match *byte {
                Byte::AnyValue => true,
                _ => false
            },
            Byte::Int(ref set1) => match *byte {
                Byte::Undefined => false,
                Byte::AnyValue => true,
                Byte::Int(ref set2) => set1.is_subset(&set2)
            }
        }
    }
}

impl fmt::Display for Byte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Byte::Undefined => String::from("??"),
            Byte::AnyValue => String::from("**"),
            Byte::Int(ref set) =>
                if set.len() == 1 {
                    format!("{:02x}", set.iter().collect::<Vec<&u8>>()[0])
                } else {
                    String::from("{}")
                },
        })
    }
}
