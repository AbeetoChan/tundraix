use num_enum::TryFromPrimitive;

use crate::value::Value;

#[derive(TryFromPrimitive, Clone)]
#[repr(u8)]
pub enum OpCode {
    Return,
    Constant,
    Nil,
    True,
    False,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal
}

#[derive(Clone)]
pub struct ValueArray {
    values: Vec<Value>
}

impl ValueArray {
    pub fn new() -> Self {
        Self {
            values: Vec::new()
        }
    }

    pub fn write_value(&mut self, value: Value) -> u8 {
        self.values.push(value);
        self.values.len() as u8 - 1        
    }

    pub fn get_value(&self, idx: u8) -> Value {
        self.values[idx as usize].clone()
    }
}

#[derive(Clone)]
pub struct Byte {
    pub byte: u8,
    pub line: usize
}

impl Byte {
    pub fn new(byte: u8, line: usize) -> Self {
        Self {
            byte,
            line
        }
    }
}

#[derive(Clone)]
pub struct Chunk {
    code: Vec<Byte>,
    value_array: ValueArray
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            value_array: ValueArray::new()
        }
    }

    pub fn write_byte(&mut self, byte: Byte) {
        self.code.push(byte);
    }

    pub fn write_value(&mut self, value: Value) -> u8 {
        self.value_array.write_value(value)
    }

    pub fn get_byte(&self, idx: usize) -> Byte {
        self.code[idx].clone()
    }

    pub fn get_value(&self, idx: u8) -> Value {
        self.value_array.get_value(idx)
    }
}