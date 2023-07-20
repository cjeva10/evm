use tiny_keccak::{Keccak, Hasher};
use crate::memory::mload_n;
use primitive_types::U256;
use crate::Memory;
use crate::utils::Stack;

const SHA3: u8 = 0x20;

fn sha3(input: &[u8], output: &mut [u8]) {
    let mut keccak = Keccak::v256();
    keccak.update(input);
    keccak.finalize(output);
}

pub fn exec(opcode: u8, stack: &mut Vec<U256>, memory: &mut Memory) {
    match opcode {
        SHA3 => {
            // load offset and size from stack
            let offset = stack.safe_pop();
            let size = stack.safe_pop();

            // load input from memory
            let input = mload_n(memory, offset, size);
            // init output
            let output = &mut [0; 32];

            sha3(&input, output);
            stack.push(U256::from_big_endian(output));
        }
        _ => panic!("Not a hash opcode"),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{TestSetup, run_test};

    #[test]
    fn sha3() {
        let setups = vec![
            TestSetup::new(
                "PUSH32 0xffffffff00000000000000000000000000000000000000000000000000000000\nPUSH1 0\nMSTORE\nPUSH1 4\nPUSH1 0\nSHA3",
                "7fffffffff000000000000000000000000000000000000000000000000000000006000526004600020",
                vec!["0x29045a592007d0c246ef02c2223570da9522d0cf0f73282c79a1bc8f0bb2c238"],
                true, 
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

}
