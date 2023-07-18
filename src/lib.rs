use primitive_types::{U256, U512};

mod arithmetic;

pub struct EvmResult {
    pub stack: Vec<U256>,
    pub success: bool,
}

// opcode aliases
const STOP: u8 = 0x00;

const POP: u8 = 0x50;
const PUSH0: u8 = 0x5f;
const PUSH1: u8 = 0x60;
const PUSH32: u8 = 0x7f;


pub fn evm(_code: impl AsRef<[u8]>) -> EvmResult {
    let mut stack: Vec<U256> = Vec::new();
    let mut pc = 0;

    let code = _code.as_ref();

    while pc < code.len() {
        let opcode = code[pc];
        pc += 1;

        match opcode {
            STOP => {
                return EvmResult {
                    stack: stack.into_iter().rev().collect(),
                    success: true,
                }
            }
            PUSH0 => stack.push(U256::zero()),
            POP => {
                stack.pop();
            }
            _ => (),
        }

        // push byte value onto the stack
        if opcode >= PUSH1 && opcode <= PUSH32 {
            let size = (opcode - PUSH1 + 1) as usize;
            stack.push(U256::from_big_endian(&code[pc..pc + size]));
            pc += size;
        }

        // arithmetic operations
        if opcode >= 0x01 && opcode <= 0x0b {
            arithmetic::do_arithmetic(opcode, &mut stack);
        }

        // comparison operations
    }

    return EvmResult {
        stack,
        success: true,
    };
}

#[cfg(test)]
mod tests {
    use super::*;


    fn run_test(
        asm: &str,
        bin: &str,
        expect_stack: Vec<&str>,
        expect_success: bool,
    ) {
        let code: Vec<u8> = hex::decode(bin).unwrap();

        let result = evm(&code);

        let mut expected_stack: Vec<U256> = Vec::new();
        for value in &expect_stack {
            expected_stack.push(U256::from_str_radix(value, 16).unwrap());
        }

        let mut matching = result.stack.len() == expect_stack.len();
        if matching {
            for i in 0..result.stack.len() {
                if result.stack[i] != expected_stack[i] {
                    matching = false;
                    break;
                }
            }
        }

        matching = matching && result.success == expect_success;

        if !matching {
            println!("Instructions: \n{}\n", asm);
            println!("Expected success: {:?}", expect_success);
            println!("Actual success: {:?}", result.success);
            println!("");

            println!("Expected stack: [");
            for v in expected_stack {
                println!("  {:#X},", v);
            }
            println!("]\n");

            println!("Actual stack: [");
            for v in result.stack {
                println!("  {:#X},", v);
            }
            println!("]\n");
        }

        assert!(matching);
    }

    #[test]
    fn stop() {
        let asm = "STOP";
        let bin = "00";
        let expect_stack = vec![];
        let expect_success = true;

        run_test(asm, bin, expect_stack, expect_success);
    }

    #[test]
    fn push0() {
        let asm = "PUSH0";
        let bin = "5f";
        let expect_stack = vec!["0x0"];
        let expect_success = true;

        run_test(asm, bin, expect_stack, expect_success);
    }

    #[test]
    fn push1() {
        let asm = "PUSH1 1";
        let bin = "6001";
        let expect_stack = vec!["0x1"];
        let expect_success = true;

        run_test(asm, bin, expect_stack, expect_success);
    }
}
