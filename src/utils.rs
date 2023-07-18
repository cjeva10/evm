use crate::{PUSH1, PUSH32};
use primitive_types::U256;

// give a better error message when popping on empty stack
pub trait Stack<T> {
    fn safe_pop(&mut self) -> T;
    fn peek(&self, depth: usize) -> Option<&T>;
    fn peek_mut(&mut self, depth: usize) -> Option<&mut T>;
}

impl Stack<U256> for Vec<U256> {
    fn safe_pop(&mut self) -> U256 {
        match self.pop() {
            Some(x) => x,
            None => panic!("Stack underflow!"),
        }
    }

    fn peek(&self, depth: usize) -> Option<&U256> {
        let len = self.len();
        if depth > len {
            return None;
        } else {
            self.get(len - depth)
        }
    }

    fn peek_mut(&mut self, depth: usize) -> Option<&mut U256> {
        let len = self.len();
        if depth > len {
            return None;
        } else {
            self.get_mut(len - depth)
        }
    }
}

pub struct ValidJumps {
    pub jumps: Vec<bool>,
}

impl ValidJumps {
    pub fn new(code: &[u8]) -> Self {
        let mut jumps: Vec<bool> = Vec::with_capacity(code.len());

        let mut i = 0;
        while i < code.len() {
            let op = code[i];
            if op == 0x5b {
                jumps.push(true);
            } else if op >= PUSH1 && op <= PUSH32 {
                let size = (op - PUSH1 + 1) as usize;
                for _ in 0..size {
                    jumps.push(false);
                }
                i += size;
                jumps.push(false);
            } else {
                jumps.push(false);
            }
            i += 1;
        }

        Self { jumps }
    }
}
