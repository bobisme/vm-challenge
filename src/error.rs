#[derive(Debug)]
pub enum Error {
    Halted,
    PoppedEmptyStack,
    ParseReg,
    ParseRegFromU8,
    ParseVal(u16),
    ParseOp(u16),
}

impl std::fmt::Display for Error {
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
