use crate::error::Error;

#[derive(Clone, Copy, Debug)]
pub struct Reg(u8);

impl Reg {
    pub const REG0: Reg = Reg(0);
    pub const REG1: Reg = Reg(1);
    pub const REG2: Reg = Reg(2);
    pub const REG3: Reg = Reg(3);
    pub const REG4: Reg = Reg(4);
    pub const REG5: Reg = Reg(5);
    pub const REG6: Reg = Reg(6);
    pub const REG7: Reg = Reg(7);

    #[inline(always)]
    pub fn index(&self) -> usize {
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

impl std::fmt::Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Val {
    Literal(u16),
    Reg(Reg),
}

impl Val {
    pub fn val(&self, registers: &[u16; 8]) -> u16 {
        match self {
            Val::Literal(x) => *x,
            Val::Reg(reg) => registers[reg.index()],
        }
    }

    pub fn is_reg(&self) -> bool {
        matches!(self, Self::Reg(..))
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

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Literal(x) => write!(f, "0x{:04x}", *x),
            Val::Reg(reg) => write!(f, "{}", reg),
        }
    }
}

pub enum Op {
    /// [0] `halt` :: Stop execution and terminate the program
    Halt,
    /// [1] `set` a b :: Set register <a> to the value of <b>
    Set(Reg, Val),
    /// [2] `push` a :: Push <a> onto the stack
    Push(Val),
    /// [3] `pop` a :: Remove the top element from the stack and write it into <a>; empty stack = error
    Pop(Reg),
    /// [4] `eq` a b c :: Set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
    Eq(Reg, Val, Val),
    /// [5] `gt` a b c :: Set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
    Gt(Reg, Val, Val),
    /// [6] `jmp` a :: Jump to <a>
    Jmp(Val),
    /// [7] `jt` a b :: If <a> is nonzero, jump to <b>
    Jt(Val, Val),
    /// [8] `jf` a b :: If <a> is zero, jump to <b>
    Jf(Val, Val),
    /// [9] `add` a b c :: Assign into <a> the sum of <b> and <c> (modulo 32768)
    Add(Reg, Val, Val),
    /// [10] `mult` a b c :: Store into <a> the product of <b> and <c> (modulo 32768)
    Mult(Reg, Val, Val),
    /// [11] `mod` a b c :: Store into <a> the remainder of <b> divided by <c>
    Mod(Reg, Val, Val),
    /// [12] `and` a b c :: Stores into <a> the bitwise and of <b> and <c>
    And(Reg, Val, Val),
    /// [13] `or` a b c :: Stores into <a> the bitwise or of <b> and <c>
    Or(Reg, Val, Val),
    /// [14] `not` a b :: Stores 15-bit bitwise inverse of <b> in <a>
    Not(Reg, Val),
    /// [15] `rmem` a b :: Read memory at address <b> and write it to <a>
    Rmem(Reg, Val),
    /// [16] `wmem` a b :: Write the value from <b> into memory at address <a>
    Wmem(Val, Val),
    /// [17] `call` a :: Write the address of the next instruction to the stack and jump to <a>
    Call(Val),
    /// [18] `ret` :: Remove the top element from the stack and jump to it; empty stack = halt
    Ret,
    /// [19] `out` a :: Write the character represented by ascii code <a> to the terminal
    Out(Val),
    /// [20] `in` a :: Read a character from the terminal and write its ascii code to <a>; it can be assumed that once input starts, it will continue until a newline is encountered; this means that you can safely read whole lines from the keyboard instead of having to figure out how to read individual characters
    In(Reg),
    /// [21] `noop` :: No operation
    Noop,
}

impl Op {
    pub fn arg_count(&self) -> usize {
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

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Halt => write!(f, "halt"),
            Op::Set(a, b) => write!(f, "set \t{},\t{}", a, b),
            Op::Push(a) => write!(f, "push\t{}", a),
            Op::Pop(a) => write!(f, "pop \t{}", a),
            Op::Eq(a, b, c) => write!(f, "eq  \t{},\t{},\t{}", a, b, c),
            Op::Gt(a, b, c) => write!(f, "gt  \t{},\t{},\t{}", a, b, c),
            Op::Jmp(a) => write!(f, "jmp \t{}", a),
            Op::Jt(a, b) => write!(f, "jt  \t{},\t{}", a, b),
            Op::Jf(a, b) => write!(f, "jf  \t{},\t{}", a, b),
            Op::Add(a, b, c) => write!(f, "add \t{},\t{},\t{}", a, b, c),
            Op::Mult(a, b, c) => write!(f, "mult\t{},\t{},\t{}", a, b, c),
            Op::Mod(a, b, c) => write!(f, "mod \t{},\t{},\t{}", a, b, c),
            Op::And(a, b, c) => write!(f, "and \t{},\t{},\t{}", a, b, c),
            Op::Or(a, b, c) => write!(f, "or  \t{},\t{},\t{}", a, b, c),
            Op::Not(a, b) => write!(f, "not \t{},\t{}", a, b),
            Op::Rmem(a, b) => write!(f, "rmem\t{},\t{}", a, b),
            Op::Wmem(a, b) => write!(f, "wmem\t{},\t{}", a, b),
            Op::Call(a) => write!(f, "call\t{}", a),
            Op::Ret => write!(f, "ret"),
            Op::Out(a) => match &a {
                Val::Literal(x) => {
                    write!(f, "out \t{}\t; {:?}", a, char::from_u32(*x as u32).unwrap())
                }
                Val::Reg(_) => write!(f, "out \t{}", a),
            },
            Op::In(a) => write!(f, "in  \t{}", a),
            Op::Noop => write!(f, "noop"),
        }
    }
}
