use crate::utils::Stack;
use crate::EvmResult;
use primitive_types::U256;

/// memory and storage opcodes 

const MLOAD: u8 = 0x51;
const MSTORE: u8 = 0x52;

pub fn exec(
    opcode: u8,
    stack: &mut Vec<U256>,
    memory: &mut [(U256, bool); 1024], 
) -> Option<EvmResult> {
    match opcode {
        MSTORE => {
            let offset = stack.safe_pop();
            let value = stack.safe_pop();
            memory[offset.as_usize()] = (value, true);
        }
        MLOAD => {
            let offset = stack.safe_pop();
            let (value, present) = memory[offset.as_usize()];
            if !present {
                panic!("This memory address is empty!");
            }
            stack.push(value);
        }
        _ => panic!("Not a memory/storage opcode"),
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::tests::{run_test, TestSetup};

    #[test]
    fn mstore() {
        let setups = vec![
            TestSetup::new(
                "PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\nPUSH1 0\nMSTORE\nPUSH1 0\nMLOAD",
                "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff600052600051",
                vec!["0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

}
