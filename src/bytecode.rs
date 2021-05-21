use crate::parser::{Literal, self};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone, Debug)]
pub enum OpCode {
    LoadConst(u16),        // (const_id)
    LoadSym(u16),          // (sym_id)
    Call(u16),             // (argc)
    Builtin(u8, u8),       // (builtin_id, argc)
    Def(u16, u16),         // (sym_id, instructions_length)
    Lambda(u16),           // (chunk_id)
    Constructor(u16, u16), // (constr_idx, to_eval)
    Tuple(u16),            // (amount)
}
impl OpCode {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Self::LoadConst(id) => {
                let mut to_ret = vec![0];
                to_ret.extend(&id.to_be_bytes());
                to_ret
            }
            Self::LoadSym(id) => {
                let mut to_ret = vec![1];
                to_ret.extend(&id.to_be_bytes());
                to_ret
            }
            Self::Call(argc) => {
                let mut to_ret = vec![2];
                to_ret.extend(&argc.to_be_bytes());
                to_ret
            }
            Self::Builtin(idx, argc) => vec![3, *idx, *argc],
            Self::Def(id, len) => {
                let mut to_ret = vec![4];
                to_ret.extend(&id.to_be_bytes());
                to_ret.extend(&len.to_be_bytes());
                to_ret
            }
            Self::Lambda(id) => {
                let mut to_ret = vec![5];
                to_ret.extend(&id.to_be_bytes());
                to_ret
            }
            Self::Constructor(idx, amount) => {
                let mut to_ret = vec![6];
                to_ret.extend(&idx.to_be_bytes());
                to_ret.extend(&amount.to_be_bytes());
                to_ret
            }
            Self::Tuple(amount) => {
                let mut to_ret = vec![7];
                to_ret.extend(&amount.to_be_bytes());
                to_ret
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub instructions: Vec<OpCode>,
    pub reference: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct Pattern {
    pub pat: parser::Pattern,
    pub to_exec: Vec<OpCode>,
}
#[derive(Clone, Debug)]
pub struct Match {
    pub expression: Vec<OpCode>,
    pub patterns: Vec<Pattern>,
}
#[derive(Clone, Debug)]
pub struct Bytecode {
    pub chunks: Vec<Chunk>,
    pub matches: Vec<Match>,
    pub symbols: Vec<String>,
    pub constants: Vec<Literal>,
    pub instructions: Vec<OpCode>,
    pub constructors: Vec<u8>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            chunks: vec![],
            symbols: vec![],
            constants: vec![],
            instructions: vec![],
            constructors: vec![],
            matches: vec![],
        }
    }
    // All numbers here are big endian
    pub fn serialize(&self) -> Vec<u8> {
        let mut to_ret = "orion".chars().into_iter().map(|c| c as u8).collect::<Vec<u8>>(); // Magic value
        to_ret.extend_from_slice(&(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32).to_be_bytes()); // Timestamp
        
        // Symbols
        to_ret.extend(&(self.symbols.len() as u16).to_be_bytes()); // Length
        self.symbols.iter().for_each(|sym| {
            sym.chars().for_each(|c| to_ret.push(c as u8));
            to_ret.push(0); // Mark termination
        });

        // Consts
        to_ret.extend(&(self.constants.len() as u16).to_be_bytes()); // Length
        self.constants.iter().for_each(|c| {
            to_ret.push(match c {
                Literal::String(_) => 0,
                Literal::Integer(_) => 1,
                Literal::Single(_) => 2,
            });

            to_ret.extend(match c {
                Literal::Integer(i) => i.to_be_bytes().to_vec(),
                Literal::Single(f) => f.to_bits().to_be_bytes().to_vec(),
                Literal::String(s) => {
                    let mut to_ex = s.chars().map(|c| c as u8).collect::<Vec<_>>();
                    to_ex.push(0); // Mark termination
                    to_ex
                }
            })
        });

        // Constructors
        to_ret.extend(&(self.constructors.len() as u16).to_be_bytes());
        to_ret.extend(self.constructors.clone());

        // Chunks
        to_ret.extend(&(self.chunks.len() as u16).to_be_bytes());
        self.chunks.iter().for_each(|chunk| {
            to_ret.extend(&(chunk.reference.len() as u16).to_be_bytes());
            chunk.reference.iter().for_each(|link| {
                to_ret.extend(&link.to_be_bytes());
            });

            let serialized = chunk.instructions.iter().map(|instr| {
                instr.serialize()
            }).flatten();
            to_ret.extend(&(serialized.clone().count() as u16).to_be_bytes());
            to_ret.extend(serialized)
        });

        // Instructions
        let serialized = self.instructions.iter().map(|instr| {
            instr.serialize()
        }).flatten();
        to_ret.extend(&(serialized.clone().count() as u16).to_be_bytes());
        to_ret.extend(serialized);

        // Match
        
        to_ret
    }
}
