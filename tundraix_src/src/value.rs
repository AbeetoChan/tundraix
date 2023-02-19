use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Nil
}

impl Value {
    pub fn as_bool(&self) -> bool {
        if let Self::Bool(v) = self {
            return *v;
        }

        unreachable!()
    }

    pub fn as_number(&self) -> f64 {
        if let Self::Number(v) = self {
            return *v;
        }

        unreachable!()
    }

    pub fn as_string(&self) -> String {
        if let Self::String(v) = self {
            return v.clone();
        }

        unreachable!()
    }

    pub fn is_bool(&self) -> bool {
        if let Self::Bool(_) = self {
            return true;
        }

        return false;
    }

    pub fn is_number(&self) -> bool {
        if let Self::Number(_) = self {
            return true;
        }

        return false;
    }

    pub fn is_string(&mut self) -> bool {
        if let Self::String(_) = self {
            return true;
        }

        return false;
    }

    pub fn is_nil(&self) -> bool {
        if let Self::Nil = self {
            return true;
        }

        return false;
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult  {
        match self {
            Value::Bool(v) => {
                write!(f, "{}", v)
            },
            Value::Number(v) => {
                write!(f, "{}", v)
            },
            Value::String(v) => {
                write!(f, "{}", v)
            },
            Value::Nil => {
                write!(f, "nil")
            }
        }
    }
}