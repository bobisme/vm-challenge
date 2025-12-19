use std::{fmt::Display, fs::File, io::Read};

const MASK: u16 = (1 << 15) - 1;
const MOD: u16 = 1 << 15;

const PRE_PROGRAMMED: &[u8] = include_bytes!("inputs.txt");

#[derive(Debug)]
enum Error {
    Halted,
    PoppedEmptyStack,
    ParseReg,
    ParseRegFromU8,
    ParseVal(u16),
    ParseOp(u16),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Halted => write!(f, "Halted"),
            Error::PoppedEmptyStack => write!(f, "Popped from empty stack"),
            Error::ParseReg => write!(f, "Failed to parse register"),
            Error::ParseRegFromU8 => write!(f, "Failed to parse register from u8"),
            Error::ParseVal(input) => write!(f, "Failed to parse value from {input}"),
            Error::ParseOp(input) => write!(f, "Failed to parse op from {input}"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Reg(u8);

impl Reg {
    fn index(&self) -> usize {
        self.0 as usize
    }
}

impl TryFrom<u8> for Reg {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 7 {
            Err(Error::ParseRegFromU8)
        } else {
            Ok(Reg(value))
        }
    }
}

impl TryFrom<u16> for Reg {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            32768..=32775 => Ok(Reg::try_from((value - 32768) as u8).unwrap()),
            _ => Err(Error::ParseReg),
        }
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{:01}", self.0)
    }
}

#[derive(Clone, Copy, Debug)]
enum Val {
    Literal(u16),
    Reg(Reg),
}

impl Val {
    fn val(&self, registers: &[u16; 8]) -> u16 {
        match self {
            Val::Literal(x) => *x,
            Val::Reg(reg) => registers[reg.index()],
        }
    }
}

impl TryFrom<u16> for Val {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0..=32767 => Ok(Self::Literal(value)),
            32768..=32775 => Ok(Self::Reg(Reg::try_from((value - 32768) as u8).unwrap())),
            _ => Err(Error::ParseVal(value)),
        }
    }
}

impl Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Literal(x) => write!(f, "{}", *x),
            Val::Reg(reg) => write!(f, "{}", reg),
        }
    }
}

enum Op {
    /// `halt` 0 :: Stop execution and terminate the program
    Halt,
    /// `set` 1 a b :: Set register <a> to the value of <b>
    Set(Reg, Val),
    /// `push` 2 a :: Push <a> onto the stack
    Push(Val),
    /// `pop` 3 a :: Remove the top element from the stack and write it into <a>; empty stack = error
    Pop(Reg),
    /// `eq` 4 a b c :: Set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
    Eq(Reg, Val, Val),
    /// `gt` 5 a b c :: Set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
    Gt(Reg, Val, Val),
    /// `jmp` 6 a :: Jump to <a>
    Jmp(Val),
    /// `jt` 7 a b :: If <a> is nonzero, jump to <b>
    Jt(Val, Val),
    /// `jf` 8 a b :: If <a> is zero, jump to <b>
    Jf(Val, Val),
    /// `add` 9 a b c :: Assign into <a> the sum of <b> and <c> (modulo 32768)
    Add(Reg, Val, Val),
    /// `mult` 10 a b c :: Store into <a> the product of <b> and <c> (modulo 32768)
    Mult(Reg, Val, Val),
    /// `mod` 11 a b c :: Store into <a> the remainder of <b> divided by <c>
    Mod(Reg, Val, Val),
    /// `and` 12 a b c :: Stores into <a> the bitwise and of <b> and <c>
    And(Reg, Val, Val),
    /// `or` 13 a b c :: Stores into <a> the bitwise or of <b> and <c>
    Or(Reg, Val, Val),
    /// `not` 14 a b :: Stores 15-bit bitwise inverse of <b> in <a>
    Not(Reg, Val),
    /// `rmem` 15 a b :: Read memory at address <b> and write it to <a>
    Rmem(Reg, Val),
    /// `wmem` 16 a b :: Write the value from <b> into memory at address <a>
    Wmem(Val, Val),
    /// `call` 17 a :: Write the address of the next instruction to the stack and jump to <a>
    Call(Val),
    /// `ret` 18 :: Remove the top element from the stack and jump to it; empty stack = halt
    Ret,
    /// `out` 19 a :: Write the character represented by ascii code <a> to the terminal
    Out(Val),
    /// `in` 20 a :: Read a character from the terminal and write its ascii code to <a>; it can be assumed that once input starts, it will continue until a newline is encountered; this means that you can safely read whole lines from the keyboard instead of having to figure out how to read individual characters
    In(Reg),
    /// `noop` 21 :: No operation
    Noop,
}

impl Op {
    fn arg_count(&self) -> usize {
        match self {
            Op::Halt | Op::Ret | Op::Noop => 0,
            Op::Push(_) | Op::Pop(_) | Op::Call(_) | Op::Out(_) | Op::In(_) | Op::Jmp(_) => 1,
            Op::Set(_, _)
            | Op::Jt(_, _)
            | Op::Jf(_, _)
            | Op::Not(_, _)
            | Op::Rmem(_, _)
            | Op::Wmem(_, _) => 2,
            Op::Eq(_, _, _)
            | Op::Gt(_, _, _)
            | Op::Add(_, _, _)
            | Op::Mult(_, _, _)
            | Op::Mod(_, _, _)
            | Op::And(_, _, _)
            | Op::Or(_, _, _) => 3,
        }
    }
}

impl TryFrom<&[u16]> for Op {
    type Error = Error;

    fn try_from(s: &[u16]) -> Result<Self, Self::Error> {
        let op = match s[0] {
            0 => Op::Halt,
            1 => Op::Set(s[1].try_into()?, s[2].try_into()?),
            2 => Op::Push(s[1].try_into()?),
            3 => Op::Pop(s[1].try_into()?),
            4 => Op::Eq(s[1].try_into()?, s[2].try_into()?, s[3].try_into()?),
            5 => Op::Gt(s[1].try_into()?, s[2].try_into()?, s[3].try_into()?),
            6 => Op::Jmp(s[1].try_into()?),
            7 => Op::Jt(s[1].try_into()?, s[2].try_into()?),
            8 => Op::Jf(s[1].try_into()?, s[2].try_into()?),
            9 => Op::Add(s[1].try_into()?, s[2].try_into()?, s[3].try_into()?),
            10 => Op::Mult(s[1].try_into()?, s[2].try_into()?, s[3].try_into()?),
            11 => Op::Mod(s[1].try_into()?, s[2].try_into()?, s[3].try_into()?),
            12 => Op::And(s[1].try_into()?, s[2].try_into()?, s[3].try_into()?),
            13 => Op::Or(s[1].try_into()?, s[2].try_into()?, s[3].try_into()?),
            14 => Op::Not(s[1].try_into()?, s[2].try_into()?),
            15 => Op::Rmem(s[1].try_into()?, s[2].try_into()?),
            16 => Op::Wmem(s[1].try_into()?, s[2].try_into()?),
            17 => Op::Call(s[1].try_into()?),
            18 => Op::Ret,
            19 => Op::Out(s[1].try_into()?),
            20 => Op::In(s[1].try_into()?),
            21 => Op::Noop,
            _ => return Err(Error::ParseOp(s[0])),
        };
        Ok(op)
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Halt => write!(f, "halt"),
            Op::Set(a, b) => write!(f, "set	{},	{}", a, b),
            Op::Push(a) => write!(f, "push	{}", a),
            Op::Pop(a) => write!(f, "pop	{}", a),
            Op::Eq(a, b, c) => write!(f, "eq	{},	{},	{}", a, b, c),
            Op::Gt(a, b, c) => write!(f, "gt	{},	{},	{}", a, b, c),
            Op::Jmp(a) => write!(f, "jmp	{}", a),
            Op::Jt(a, b) => write!(f, "jt	{},	{}", a, b),
            Op::Jf(a, b) => write!(f, "jf	{},	{}", a, b),
            Op::Add(a, b, c) => write!(f, "add	{},	{},	{}", a, b, c),
            Op::Mult(a, b, c) => write!(f, "mult	{},	{},	{}", a, b, c),
            Op::Mod(a, b, c) => write!(f, "mod	{},	{},	{}", a, b, c),
            Op::And(a, b, c) => write!(f, "and	{},	{},	{}", a, b, c),
            Op::Or(a, b, c) => write!(f, "or	{},	{},	{}", a, b, c),
            Op::Not(a, b) => write!(f, "not	{},	{}", a, b),
            Op::Rmem(a, b) => write!(f, "rmem	{},	{}", a, b),
            Op::Wmem(a, b) => write!(f, "wmem	{},	{}", a, b),
            Op::Call(a) => write!(f, "call	{}", a),
            Op::Ret => write!(f, "ret"),
            Op::Out(a) => match &a {
                Val::Literal(x) => {
                    write!(
                        f,
                        "out	{}\t\t// {:?}",
                        a,
                        char::from_u32(*x as u32).unwrap()
                    )
                }
                Val::Reg(_) => write!(f, "out	{}", a),
            },
            Op::In(a) => write!(f, "in	{}", a),
            Op::Noop => write!(f, "noop"),
        }
    }
}

#[derive(Clone, Default)]
struct Machine {
    registers: [u16; 8],
    stack: Vec<u16>,
    mem: Vec<u16>,
    mem_offset: usize,
    /// Pre-programmed input index
    pp: usize,
}

impl Machine {
    fn jump_to_addr(&mut self, addr: u16) {
        self.mem_offset = addr as usize
    }

    fn jump(&mut self, val: Val) {
        self.jump_to_addr(self.val(val));
    }

    fn val(&self, val: Val) -> u16 {
        val.val(&self.registers)
    }

    fn set_lit(&mut self, reg: Reg, val: u16) {
        self.registers[reg.index()] = val;
    }

    fn set(&mut self, reg: Reg, val: Val) {
        self.registers[reg.index()] = self.val(val);
    }

    fn apply(&mut self, op: Op) -> Result<bool, Error> {
        let jumped = match op {
            Op::Halt => return Err(Error::Halted),
            Op::Set(a, b) => {
                self.set(a, b);
                false
            }
            Op::Push(a) => {
                self.stack.push(self.val(a));
                false
            }
            Op::Pop(a) => {
                let val = self.stack.pop().unwrap();
                self.set_lit(a, val);
                false
            }
            Op::Eq(a, b, c) => {
                self.registers[a.index()] = if self.val(b) == self.val(c) { 1 } else { 0 };
                false
            }
            Op::Gt(a, b, c) => {
                self.registers[a.index()] = if self.val(b) > self.val(c) { 1 } else { 0 };
                false
            }
            Op::Jmp(val) => {
                self.jump(val);
                true
            }
            Op::Jt(a, b) => {
                if self.val(a) != 0 {
                    self.jump(b);
                    true
                } else {
                    false
                }
            }
            Op::Jf(a, b) => {
                if self.val(a) == 0 {
                    self.jump(b);
                    true
                } else {
                    false
                }
            }
            Op::Add(a, b, c) => {
                self.set_lit(
                    a,
                    ((self.val(b) as u32 + self.val(c) as u32) % MOD as u32) as u16,
                );
                false
            }
            Op::Mult(a, b, c) => {
                self.set_lit(
                    a,
                    ((self.val(b) as u32 * self.val(c) as u32) % MOD as u32) as u16,
                );
                false
            }
            Op::Mod(a, b, c) => {
                self.set_lit(a, self.val(b) % self.val(c));
                false
            }
            Op::And(a, b, c) => {
                self.set_lit(a, self.val(b) & self.val(c));
                false
            }
            Op::Or(a, b, c) => {
                self.set_lit(a, self.val(b) | self.val(c));
                false
            }
            Op::Not(a, b) => {
                self.set_lit(a, MASK ^ self.val(b));
                false
            }
            Op::Rmem(a, b) => {
                self.set_lit(a, self.mem[self.val(b) as usize]);
                false
            }
            Op::Wmem(a, b) => {
                let addr = self.val(a) as usize;
                if self.mem.len() < addr {
                    self.mem.resize(addr - 1, 0);
                }
                self.mem[addr] = self.val(b);
                false
            }
            Op::Call(a) => {
                self.mem_offset += 2;
                self.stack.push(self.mem_offset as u16);
                self.jump(a);
                true
            }
            Op::Ret => {
                let Some(popped) = self.stack.pop() else {
                    return Err(Error::PoppedEmptyStack);
                };
                self.jump_to_addr(popped);
                true
            }
            Op::Out(a) => {
                print!("{}", self.val(a) as u8 as char);
                false
            }
            Op::In(a) => {
                if self.pp < PRE_PROGRAMMED.len() {
                    let c = PRE_PROGRAMMED[self.pp];
                    self.set_lit(a, c as u16);
                    print!("{}", char::from_u32(c as u32).unwrap());
                    self.pp += 1;
                } else {
                    let mut b = [0u8; 1];
                    std::io::stdin()
                        .lock()
                        .read_exact(&mut b)
                        .expect("failed to read input char");
                    self.set_lit(a, b[0] as u16);
                }
                false
            }
            Op::Noop => false,
        };
        Ok(jumped)
    }

    fn run(&mut self) {
        loop {
            let op = match Op::try_from(&self.mem[self.mem_offset..]) {
                Ok(op) => op,
                Err(err) => panic!(
                    "failed to parse opcode at offset {}: {err:?}",
                    self.mem_offset
                ),
            };
            // println!("{op}");
            let offset = 1 + op.arg_count();
            let jumped = self.apply(op).unwrap();
            if !jumped {
                self.mem_offset += offset;
            }
        }
    }
}

fn decompile(mem: &[u16]) {
    let mut offset = 0;
    let mut in_data = false;
    while offset < mem.len() {
        let op = match Op::try_from(&mem[offset..]) {
            Ok(op) => {
                if in_data {
                    println!("\n\n.ops:\n");
                    in_data = false;
                }
                op
            }
            Err(_err) => {
                if !in_data {
                    in_data = true;
                    println!("\n.data:\n");
                }
                print!("{}", ((mem[offset] >> 8) as u8).escape_ascii());
                print!("{}", ((mem[offset] & 0xFF) as u8).escape_ascii());
                offset += 1;
                continue;
            }
        };
        println!("\t\t{op}");
        offset += 1 + op.arg_count();
    }
}

fn load_mem() -> Vec<u16> {
    let mut f = File::open("./challenge.bin").unwrap();
    let mut rom_data: Vec<u8> = Vec::with_capacity(f.metadata().unwrap().len() as usize);
    f.read_to_end(&mut rom_data).unwrap();
    let (chunks, _) = rom_data.as_chunks::<2>();
    chunks.iter().cloned().map(u16::from_le_bytes).collect()
}

fn main() {
    match std::env::args().nth(1) {
        Some(s) => match s.as_str() {
            "decompile" => {
                let mem = load_mem();
                decompile(&mem);
            }
            _ => println!("what"),
        },
        _ => {
            let mem = load_mem();
            let mut machine = Machine {
                mem,
                ..Default::default()
            };
            machine.run();
        }
    }
}
