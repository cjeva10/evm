use crate::utils::Stack;
use crate::EvmResult;
use primitive_types::U256;
use std::collections::HashMap;

/// memory opcodes

const MLOAD: u8 = 0x51;
const MSTORE: u8 = 0x52;
const MSTORE8: u8 = 0x53;

fn mstore(memory: &mut HashMap<U256, u8>, offset: U256, value: U256) {
    let mut bytes = [0; 32];
    value.to_big_endian(&mut bytes);
    for (i, byte) in bytes.iter().enumerate() {
        let idx = offset + U256::from(i);
        let addr = memory.entry(idx).or_insert(0);
        *addr = *byte;
    }
}

fn mload(memory: &mut HashMap<U256, u8>, offset: U256) -> U256 {
    // get the next 32 bytes in memory
    let mut bytes = [0; 32];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let idx = offset + U256::from(i);
        *byte = *memory.entry(idx).or_insert(0);
    }

    U256::from_big_endian(&bytes)
}

pub fn exec(
    opcode: u8,
    stack: &mut Vec<U256>,
    memory: &mut HashMap<U256, u8>,
) -> Option<EvmResult> {
    match opcode {
        MSTORE => {
            let offset = stack.safe_pop();
            let value = stack.safe_pop();

            mstore(memory, offset, value);
        }
        MLOAD => {
            let offset = stack.safe_pop();

            let value = mload(memory, offset);
            stack.push(value);
        }
        MSTORE8 => {
            let offset = stack.safe_pop();
            let value = stack.safe_pop();
            let addr = memory.entry(offset).or_insert(0);
            *addr = value.byte(0);
        }
        _ => panic!("Not a memory opcode"),
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
            TestSetup::new(
                "PUSH32 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20\nPUSH1 0\nMSTORE\nPUSH1 31\nMLOAD",
                "7f0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20600052601f51",
                vec!["0x2000000000000000000000000000000000000000000000000000000000000000"],
                true,
            ),
            TestSetup::new(
                "PUSH1 0xff\nPUSH1 31\nMSTORE8\nPUSH1 0\nMLOAD",
                "60ff601f53600051",
                vec!["0xff"],
                true
            )
        ];

        for setup in setups {
            run_test(setup);
        }
    }
}
