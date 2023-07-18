use crate::utils::Stack;
use primitive_types::U256;

const DUP1: u8 = 0x80;
const DUP16: u8 = 0x8f;
const SWAP1: u8 = 0x90;
const SWAP16: u8 = 0x9f;

pub fn exec(opcode: u8, stack: &mut Vec<U256>) {
    println!("opcode = {:x?}", opcode);
    if opcode >= DUP1 && opcode <= DUP16 {
        let depth: usize = (opcode - DUP1 + 1).into();
        let left = stack.peek(depth).unwrap();
        stack.push(*left);
    } else if opcode >= SWAP1 && opcode <= SWAP16 {
        let top = stack.safe_pop();
        println!("top = {}", top);

        let depth: usize = (opcode - SWAP1 + 1).into();
        println!("depth = {}", depth);
        let deep: &mut U256 = stack.peek_mut(depth).unwrap();
        println!("deep = {}", deep);

        // copy the deep item to push later
        let new_top = deep.clone();
        println!("new_top = {}", new_top);

        // change the item deep in the stack
        *deep = top;

        // push the new top of the stack
        stack.push(new_top);
        println!("");
    } else {
        panic!("Opcode is not a DUP or SWAP");
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{run_test, TestSetup};

    #[test]
    fn dup() {
        let setups = vec![
            TestSetup::new("PUSH1 1\nDUP1", "600180", vec!["0x1", "0x1"], true),
            TestSetup::new("PUSH1 2\nPUSH1 1\nDUP2", "6002600181", vec!["0x2", "0x1", "0x2"], true),
            TestSetup::new("PUSH1 2\nPUSH1 1\nPUSH1 1\nDUP3", "60026001600182", vec!["0x2", "0x1", "0x1", "0x2"], true),
            TestSetup::new("PUSH1 2\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nPUSH1 1\nDUP16", "60026001600160016001600160016001600160016001600160016001600160018f", vec!["0x2", "0x1", "0x1","0x1","0x1","0x1","0x1","0x1","0x1","0x1","0x1","0x1","0x1","0x1","0x1","0x1","0x2"], true),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn swap() {
        let setups = vec![
            TestSetup::new("PUSH1 1\nPUSH1 2\nSWAP1", "6001600290", vec!["0x2", "0x1"], true),
            TestSetup::new("PUSH1 0..2\nSWAP2", "60006001600291", vec!["0x2", "0x1", "0x0"], true),
            TestSetup::new("PUSH1 0..3\nSWAP3", "600060016002600392", vec!["0x3", "0x1", "0x2", "0x0"], true),
            TestSetup::new("PUSH1 0..4\nSWAP4", "6000600160026003600493", vec!["0x4", "0x1", "0x2","0x3", "0x0"], true),
            TestSetup::new("PUSH1 0..16\nSWAP16", "6000600160026003600460056006600760086009600a600b600c600d600e600f60109f", vec!["0x10", "0x1", "0x2","0x3", "0x4", "0x5", "0x6", "0x7", "0x8", "0x9", "0xa", "0xb", "0xc", "0xd", "0xe", "0xf", "0x0"], true),
        ];

        for setup in setups {
            run_test(setup);
        }
    }
}
