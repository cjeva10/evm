use crate::utils::Stack;
use crate::{EvmResult, Memory};
use primitive_types::U256;
use std::collections::HashMap;

/// memory opcodes

const MLOAD: u8 = 0x51;
const MSTORE: u8 = 0x52;
const MSTORE8: u8 = 0x53;
const MSIZE: u8 = 0x59;

fn set_msize(memory: &mut Memory, highest_access: U256) {
    // memsize = offset + 31 + 32 - (offset + 31 + 32) % 32
    // find the smallest multiple of 32 above the last byte we accessed
    let highest_byte_accessed = highest_access.overflowing_add(U256::from(32)).0;
    if memory.size < highest_byte_accessed {
        memory.size = highest_byte_accessed
            .overflowing_sub(
                highest_byte_accessed
                    .checked_rem(U256::from(32))
                    .unwrap_or(U256::zero()),
            )
            .0;
    }
}

fn mstore(memory: &mut Memory, offset: U256, value: U256) {
    let mut bytes = [0; 32];
    value.to_big_endian(&mut bytes);
    let mut idx = U256::zero();
    for (i, byte) in bytes.iter().enumerate() {
        idx = offset + U256::from(i);
        let addr = memory.data.entry(idx).or_insert(0);
        *addr = *byte;
    }

    set_msize(memory, idx);
}

fn mload(memory: &mut Memory, offset: U256) -> U256 {
    // get the next 32 bytes in memory
    let mut bytes = [0; 32];
    let mut idx = U256::zero();
    for (i, byte) in bytes.iter_mut().enumerate() {
        idx = offset.overflowing_add(U256::from(i)).0;
        *byte = *memory.data.entry(idx).or_insert(0);
    }

    set_msize(memory, idx);

    U256::from_big_endian(&bytes)
}

pub fn exec(opcode: u8, stack: &mut Vec<U256>, memory: &mut Memory) -> Option<EvmResult> {
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
            let addr = memory.data.entry(offset).or_insert(0);
            *addr = value.byte(0);
            set_msize(memory, offset);
        }
        MSIZE => {
            stack.push(memory.size);
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

    #[test]
    fn msize() {
        let setups = vec![
            TestSetup::new(
                "PUSH1 0\nMLOAD\nPOP\nMSIZE",
                "6000515059",
                vec!["0x20"],
                true,
            ),
            TestSetup::new(
                "PUSH1 0x39\nMLOAD\nPOP\nMSIZE",
                "6039515059",
                vec!["0x60"],
                true,
            ),
            TestSetup::new(
                "PUSH1 0xff\nPUSH1 0xff\nMSTORE8\nMSIZE",
                "60ff60ff5359",
                vec!["0x100"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }
}
