use std::fmt::Write;

use std::convert::TryFrom;

use crate::chunk::{Chunk, OpCode, Byte};
use crate::error::{ErrorResult, Error};
use crate::value::Value;

type Table = std::collections::HashMap<String, Value>;
type PrintFn = fn(String) -> ErrorResult<()>;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    current_instruction: Byte,
    stack: [Value; 256],
    stack_top: usize,
    globals: Table,
    print_fn: PrintFn
}

impl VM {
    pub fn new(print_fn: PrintFn) -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            current_instruction: Byte::new(0, 0),
            stack: [(); 256].map(|_| Value::Nil),
            stack_top: 0,
            globals: std::collections::HashMap::new(),
            print_fn
        }
    }

    pub fn pop_value(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top].clone()
    }

    pub fn push_value(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    pub fn interpret(&mut self, chunk: Chunk) -> ErrorResult<()> {
        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }

    fn read_byte(&mut self) -> Byte {
        let opcode = self.chunk.get_byte(self.ip);
        self.ip += 1;
        opcode
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.chunk.get_value(byte.byte)
    }
    
    fn peek(&mut self, distance: usize) -> Value {
        self.stack[self.stack_top - 1 - distance].clone()
    }

    fn is_falsey(&mut self, value: Value) -> bool {
        value.is_nil() || (value.is_bool() && !value.as_bool())
    }

    fn concat(&mut self) {
        let b = self.pop_value().as_string();
        let a = self.pop_value().as_string();
        let concat = format!("{}{}", a, b);
        self.push_value(Value::String(concat));
    }

    fn read_string(&mut self) -> String {
        self.read_constant().to_string()
    }

    #[allow(unused_must_use)]
    fn error(&mut self, message: Error) -> ErrorResult<()> {
        let mut error_string = Error::new();
        error_string.write_str(format!("[line {}] Error: ", self.current_instruction.line).as_str());
        error_string.write_str(message.as_str());

        Err(error_string)
    }

    pub fn run(&mut self) -> ErrorResult<()> {
        macro_rules! binop {
            ($value_type: ident, $op: tt) => {{
                if !self.peek(0).is_number() || !self.peek(1).is_number() {
                    return self.error(Error::from("Operands must be numbers."))
                }

                let b = self.pop_value().as_number();
                let a = self.pop_value().as_number();
                self.push_value(Value::$value_type(a $op b))
            }}
        }

        loop {
            self.current_instruction = self.read_byte();

            match OpCode::try_from(self.current_instruction.byte).unwrap() {
                OpCode::Return => {
                    break;
                },
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.push_value(constant);
                },
                OpCode::Negate => {
                    if !self.peek(0).is_number() {
                        return self.error(Error::from("Operand(s) must be a number."))
                    }

                    let value = self.pop_value();
                    self.push_value(Value::Number(-value.as_number()))
                },
                OpCode::Add => {
                    if self.peek(0).is_string() && self.peek(1).is_string() {
                        self.concat();
                    } else if self.peek(0).is_number() && self.peek(1).is_number() {
                        let b = self.pop_value().as_number();
                        let a = self.pop_value().as_number();
                        self.push_value(Value::Number(a + b));
                    } else {
                        return self.error(Error::from("Invalid operands."))
                    }
                },
                OpCode::Subtract => binop!(Number, -),
                OpCode::Multiply => binop!(Number, *),
                OpCode::Divide => binop!(Number, /),
                OpCode::Nil => self.push_value(Value::Nil),
                OpCode::True => self.push_value(Value::Bool(true)),
                OpCode::False => self.push_value(Value::Bool(false)),
                OpCode::Not => {
                    let popped = self.pop_value();
                    let is_falsey = self.is_falsey(popped);
                    self.push_value(Value::Bool(is_falsey))
                },
                OpCode::Equal => {
                    let b = self.pop_value();
                    let a = self.pop_value();
                    self.push_value(Value::Bool(a == b));
                },
                OpCode::Greater => binop!(Bool, >),
                OpCode::Less => binop!(Bool, <),
                OpCode::Print => {
                    let popped = self.pop_value();
                    self.print_fn.clone()(format!("{}\n", popped))?;
                },
                OpCode::Pop => {
                    self.pop_value();
                },
                OpCode::DefineGlobal => {
                    let name = self.read_string();
                    let value = self.peek(0);
                    self.globals.insert(name, value);
                    self.pop_value();
                },
                OpCode::GetGlobal => {
                    let name = self.read_string();
                    if !self.globals.contains_key(&name) {
                        return self.error(format!("Undefined variable {}", name));
                    }
                    let value = self.globals.get(&name).unwrap();
                    self.push_value(value.clone());
                },
                OpCode::SetGlobal => {
                    let name = self.read_string();
                    if !self.globals.contains_key(&name) {
                        return self.error(format!("Undefined variable {}", name));
                    }
                    *self.globals.get_mut(&name).unwrap() = self.peek(0)
                }
            }
        }

        Ok(())
    }
}