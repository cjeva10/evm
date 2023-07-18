use primitive_types::U256;

// give a better error message when popping on empty stack
pub trait Stack<T> {
    fn safe_pop(&mut self) -> T;
    fn peek(&self, depth: usize) -> Option<&T>;
    fn peek_mut(&mut self, depth: usize) -> Option<&mut T>;
}

impl Stack<U256> for Vec<U256> {
    fn safe_pop(&mut self) -> U256 {
        match  self.pop() {
            Some(x) => x,
            None => panic!("Stack underflow!"),
        }
    }

    fn peek(&self, depth: usize) -> Option<&U256> {
        let len = self.len();
        if depth > len {
            return None;
        } else {
            self.get(len-depth)
        }
    }

    fn peek_mut(&mut self, depth: usize) -> Option<&mut U256> {
        let len = self.len();
        if depth > len {
            return None
        } else {
            self.get_mut(len-depth)
        }
    }
}

