use std::fmt::Display;
use std::convert::TryFrom;

pub struct Stack<T>(std::collections::LinkedList<T>);

impl<T: Clone> Stack<T> {
    pub fn new() -> Self {
        Self(std::collections::LinkedList::new())
    }

    pub fn push(&mut self, e: &T) {
        self.0.push_back(e.clone())
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop_back()
    }

    pub fn dup(&mut self) -> Result<(),()> {
        let front = match self.0.back() {
            Some(e) => e.clone(),
            None => return Err(()),
        };
        self.push(&front);
        Ok(())
    }
}

#[allow(non_camel_case_types)]
pub enum ScriptOp {
    OP_FALSE,
    NA(u8),
    OP_TRUE,
    OP_DUP,
    OP_EQUAL,
    OP_VERIFY,
    OP_HASH,
    OP_CHECKSIG,
}

#[derive(Debug)]
pub struct WrongOPCode;

impl Display for WrongOPCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Wrong OP code")
    }
}

impl std::error::Error for WrongOPCode {}

impl TryFrom<u8> for ScriptOp {
    type Error = WrongOPCode;

    fn try_from(e: u8) -> Result<Self, Self::Error> {
        match e {
            0x00 => Ok(Self::OP_FALSE),
            a @ 0x01..=0x4B => Ok(Self::NA(a)),
            0x50 => Ok(Self::OP_TRUE),
            0x64 => Ok(Self::OP_DUP),
            0x78 => Ok(Self::OP_EQUAL),
            0x8C => Ok(Self::OP_VERIFY),
            0xA0 => Ok(Self::OP_HASH),
            0xAA => Ok(Self::OP_CHECKSIG),
            _ => Err(WrongOPCode),
        }
    }
}

pub fn verify_script(_s: &[u8]) -> bool {
    true
}
