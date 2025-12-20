use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Write},
    rc::Rc,
};

use crate::{
    error::Error,
    op::{Op, Reg, Val},
};

const MASK: u16 = (1 << 15) - 1;
const MOD: u16 = 1 << 15;

#[derive(Default)]
pub struct Machine {
    registers: [u16; 8],
    stack: Vec<u16>,
    mem: Vec<u16>,
    mem_offset: usize,
    // DEBUG
    // Pre-programmed input index
    pp: Rc<[u8]>,
    pp_idx: usize,
    //
    trace_file: Option<BufWriter<File>>,
    // Watch addresses
    watches: HashMap<u16, String>,
    input_log: String,
}

impl Machine {
    pub fn new(mem: Vec<u16>) -> Self {
        Self {
            mem,
            ..Default::default()
        }
    }

    pub fn set_script(&mut self, script: &[u8]) {
        self.pp = script
            .split_inclusive(|c| *c == b'\n')
            .filter(|line| !line.starts_with(b"//"))
            .flatten()
            .cloned()
            .collect();
    }

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
        if let Some(trace) = &mut self.trace_file {
            writeln!(trace, "; set {reg} = {val}").unwrap();
            // writeln!(trace, "// REGISTERS: {:?}", self.registers).unwrap();
        }
        self.registers[reg.index()] = val;
    }

    fn set(&mut self, reg: Reg, val: Val) {
        self.set_lit(reg, self.val(val));
    }

    fn apply(&mut self, op: Op) -> Result<bool, Error> {
        if let Some(trace) = &mut self.trace_file {
            writeln!(trace, "{op}").unwrap()
        }
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
                self.set_lit(a, if self.val(b) == self.val(c) { 1 } else { 0 });
                // self.registers[a.index()] = if self.val(b) == self.val(c) { 1 } else { 0 };
                false
            }
            Op::Gt(a, b, c) => {
                self.set_lit(a, if self.val(b) > self.val(c) { 1 } else { 0 });
                // self.registers[a.index()] = if self.val(b) > self.val(c) { 1 } else { 0 };
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
                let addr = self.val(b) as usize;
                let mem_val = self.mem[addr];
                self.set_lit(a, mem_val);
                if self.watches.contains_key(&(addr as u16)) {
                    println!(
                        "DEBUG: read {} addr {addr} = {mem_val}",
                        self.watches[&(addr as u16)]
                    );
                }
                false
            }
            Op::Wmem(a, b) => {
                let addr = self.val(a) as usize;
                if self.mem.len() < addr {
                    self.mem.resize(addr - 1, 0);
                }
                let val = self.val(b);
                self.mem[addr] = val;
                if self.watches.contains_key(&(addr as u16)) {
                    println!(
                        "DEBUG: write {} addr {addr} = {val}",
                        self.watches[&(addr as u16)]
                    );
                }
                false
            }
            Op::Call(a) => {
                self.mem_offset += 2;
                self.stack.push(self.mem_offset as u16);
                let addr = self.val(a);
                self.jump_to_addr(addr);
                if let Some(trace) = &mut self.trace_file
                    && let Val::Reg(_) = a
                {
                    writeln!(trace, "; register {a} = 0x{addr:04x}").unwrap();
                }
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
                if self.pp_idx < self.pp.len() {
                    let c = self.pp[self.pp_idx];
                    self.set_lit(a, c as u16);
                    print!("{}", char::from_u32(c as u32).unwrap());
                    self.pp_idx += 1;
                } else {
                    let mut b = [0u8; 1];
                    std::io::stdin()
                        .lock()
                        .read_exact(&mut b)
                        .expect("failed to read input char");
                    self.input_log.push(char::from_u32(b[0] as u32).unwrap());
                    let code = "use teleporter\n";
                    if self.input_log.len() >= code.len()
                        && &self.input_log[(self.input_log.len() - code.len())..] == code
                    {
                        if let Some(trace) = &mut self.trace_file {
                            writeln!(trace, ";; USING TELEPORTER").unwrap();
                        }
                        // self.set_eighth_register(399);
                        self.set_eighth_register(1);
                    }
                    self.set_lit(a, b[0] as u16);
                }
                false
            }
            Op::Noop => false,
        };
        Ok(jumped)
    }

    pub fn run(&mut self) {
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
            let jumped = match self.apply(op) {
                Ok(jumped) => jumped,
                Err(Error::Halted) => {
                    return self.stop();
                }
                Err(err) => panic!("{:?}", err),
            };
            if !jumped {
                self.mem_offset += offset;
            }
        }
    }

    pub fn stop(&mut self) {
        println!("Game Over");
        if let Some(trace) = &mut self.trace_file {
            let _ = trace.flush();
        }
    }

    pub fn set_trace_out(&mut self, filename: &str) {
        let trace_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .unwrap();
        self.trace_file = Some(BufWriter::new(trace_file));
    }

    pub fn watch(&mut self, addr: u16, name: &str) {
        self.watches.insert(addr, name.to_owned());
    }

    pub fn set_eighth_register(&mut self, val: u16) {
        self.registers[7] = val;
    }
}
