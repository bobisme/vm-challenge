use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use crate::machine::Machine;

pub mod error;
pub mod machine;
pub mod op;

const PRE_PROGRAMMED: &[u8] = include_bytes!("inputs.txt");

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
                print!("/* 0x{addr:04x} */\t{op}");
                if let Some(list) = addr_annos {
                    list.iter().for_each(|a| {
                        if let Annotation::Comment(text) = a {
                            println!("\t\t\t; {text}");
                        }
                    })
                } else {
                    println!();
                }
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
            let mut machine = Machine::new(mem);
            machine.set_script(PRE_PROGRAMMED);
            machine.set_trace_out("run.trace");
            machine.run();
        }
    }
}
