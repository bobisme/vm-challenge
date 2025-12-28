use std::{
    collections::HashMap,
    fmt::Write,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    rc::Rc,
};

use crate::machine::{MAX_U15, MOD, Machine};

pub mod error;
pub mod machine;
pub mod op;

const BIN_PATH: &str = "challenge.bin";

#[derive(Clone, Debug)]
enum Annotation {
    Comment(String),
    Label(String),
}

enum AnnotationSection {
    Unknown,
    Comments,
    Labels,
}

#[derive(Debug)]
struct ParseAnnotationsError;
impl std::fmt::Display for ParseAnnotationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse annotation file")
    }
}
impl std::error::Error for ParseAnnotationsError {}

type Annotations = HashMap<u16, Vec<Annotation>>;

fn load_annotations(path: &Path) -> Result<Option<Annotations>, Box<dyn std::error::Error>> {
    if !std::fs::exists(path)? {
        return Ok(None);
    }
    let reader = BufReader::new(File::open(path)?);
    let mut section = AnnotationSection::Unknown;
    let mut annotations = Annotations::new();
    for line in reader.lines() {
        match line?.as_str() {
            "[comments]" => section = AnnotationSection::Comments,
            "[labels]" => section = AnnotationSection::Labels,
            l => {
                let Some((left, right)) = l.split_once(" = ") else {
                    continue;
                };
                let addr = u16::from_str_radix(left.trim_start_matches("0x"), 16)?;
                let text = right.strip_prefix("\"").and_then(|s| s.strip_suffix("\""));
                let Some(text) = text else {
                    continue;
                };
                match section {
                    AnnotationSection::Comments => {
                        annotations
                            .entry(addr)
                            .and_modify(|v| v.push(Annotation::Comment(text.to_owned())))
                            .or_insert_with(|| vec![Annotation::Comment(text.to_owned())]);
                    }
                    AnnotationSection::Labels => {
                        annotations
                            .entry(addr)
                            .and_modify(|v| v.push(Annotation::Label(text.to_owned())))
                            .or_insert_with(|| vec![Annotation::Label(text.to_owned())]);
                    }
                    AnnotationSection::Unknown => return Err(ParseAnnotationsError.into()),
                };
            }
        }
    }
    Ok(Some(annotations))
}

fn decompile(mem: &[u16]) {
    let annotations = load_annotations(Path::new("annotations.ini")).unwrap();
    let mut addr = 0;
    let mut in_data = false;
    while addr < mem.len() {
        match crate::op::Op::try_from(&mem[addr..]) {
            Ok(op) => {
                let mut out_line = String::new();
                if in_data {
                    // println!("\n\nops:");
                    println!("\n");
                    in_data = false;
                }
                let addr_annos = annotations.as_ref().and_then(|m| m.get(&(addr as u16)));
                if let Some(list) = addr_annos {
                    list.iter().for_each(|a| {
                        if let Annotation::Label(text) = a {
                            println!("{text}:");
                        }
                    })
                }
                write!(out_line, "/* 0x{addr:04x} */ {op}").unwrap();
                if let Some(list) = addr_annos {
                    list.iter().for_each(|a| {
                        if let Annotation::Comment(text) = a {
                            for _ in out_line.len()..40 {
                                write!(out_line, " ").unwrap();
                            }
                            write!(out_line, "; {text}").unwrap();
                        }
                    })
                }
                println!("{out_line}");
                addr += 1 + op.arg_count();
            }
            Err(_err) => {
                if !in_data {
                    in_data = true;
                    // println!("\ndata:");
                    print!("; binary data omitted");
                }
                // print!("{}", ((mem[offset] >> 8) as u8).escape_ascii());
                // print!("{}", ((mem[offset] & 0xFF) as u8).escape_ascii());
                addr += 1;
                continue;
            }
        };
    }
}

fn calc_reg_8() {
    /// Non-literal implementation of `recursive_function` with memoization.
    /// See `./teleporter.py` for notes and derivation.
    fn recfn(r0: u16, r1: u16, r7: u16, cache: &mut HashMap<(u16, u16), u16>) -> u16 {
        let key = (r0, r1);
        if cache.contains_key(&key) {
            return *cache.get(&key).unwrap();
        }
        let res = if r0 == 0 {
            (r1 + 1) % MOD
        } else if r1 == 0 {
            recfn(r0 - 1, r7, r7, cache)
        } else {
            recfn(r0 - 1, recfn(r0, r1 - 1, r7, cache), r7, cache)
        };
        cache.insert(key, res);
        res
    }

    for r7 in 0..=MAX_U15 {
        print!("{r7}.");
        let mut cache = HashMap::<(u16, u16), u16>::new();
        if recfn(4, 1, r7, &mut cache) == 6 {
            println!("FOUND r7 = {r7}");
            return;
        }
    }
    println!("NOTHING FOUND");
}

fn load_mem(path: &Path) -> Vec<u16> {
    let rom_data = std::fs::read(path).unwrap();
    let (chunks, _) = rom_data.as_chunks::<2>();
    chunks.iter().cloned().map(u16::from_le_bytes).collect()
}

fn main() {
    let args: Rc<[String]> = std::env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("decompile") => {
            let mem = load_mem(Path::new(BIN_PATH));
            decompile(&mem);
        }
        Some("reg8") => calc_reg_8(),
        None | Some("run") => {
            let mem = load_mem(Path::new(BIN_PATH));
            let mut machine = Machine::new(mem);
            if let Some((i, _)) = args
                .iter()
                .enumerate()
                .find(|(_, x)| x.as_str() == "--script")
            {
                let script =
                    std::fs::read(args.get(i + 1).expect("usage: --script <file>")).unwrap();
                machine.set_script(&script);
            }
            if args.contains(&"--trace".to_owned()) {
                machine.set_trace_out("run.trace");
            }
            if args.contains(&"--hack-teleporter".to_owned()) {
                println!("HACKS ENABLED");
                machine.hack_teleporter();
            }
            machine.run();
        }
        Some(cmd) => println!("Unknown command: {cmd}"),
    }
}
