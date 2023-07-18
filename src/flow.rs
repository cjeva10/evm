use crate::utils::Stack;
use crate::EvmResult;
use primitive_types::U256;

/// control flow opcode implementations

const JUMP: u8 = 0x56;
const JUMPI: u8 = 0x57;
const JUMPDEST: u8 = 0x5b;

fn jump(
    stack: &mut Vec<U256>,
    pc: &mut usize,
    code: &[u8],
    jumps: &Vec<bool>,
    dest: U256,
) -> Option<EvmResult> {
    println!("dest = {}", dest);

    if dest >= U256::from(code.len()) {
        panic!("Jump destination is out of bounds");
    }

    *pc = dest.as_usize();

    println!("{}", jumps[*pc]);
    if let Some(jump) = jumps.get(*pc) {
        println!("{}", jump);
        if !jump {
            return Some(EvmResult {
                stack: stack.clone(),
                success: false,
            });
        }
    }

    None
}

pub fn exec(
    opcode: u8,
    stack: &mut Vec<U256>,
    pc: &mut usize,
    code: &[u8],
    jumps: &Vec<bool>,
) -> Option<EvmResult> {
    match opcode {
        JUMP => {
            let dest = stack.safe_pop();
            return jump(stack, pc, code, jumps, dest);
        }
        JUMPI => {
            let dest = stack.safe_pop();
            let cond = stack.safe_pop();
            if cond != U256::zero() {
                return jump(stack, pc, code, jumps, dest);
            }
        }
        JUMPDEST => (),
        _ => panic!("Not a control flow opcode"),
    }

    return None;
}

#[cfg(test)]
mod tests {
    use crate::tests::{run_test, TestSetup};

    #[test]
    fn jump() {
        let setups = vec![
            TestSetup::new(
                "PUSH1 5\nJUMP\nPUSH1 1\nJUMPDEST\nPUSH1 2",
                "60055660015b6002",
                vec!["0x2"],
                true,
            ),
            TestSetup::new("PUSH1 3\nJUMP\nPUSH1 1", "6003566001", vec![], false),
            TestSetup::new(
                "PUSH1 4\nJUMP\nPUSH1 0x5b\nPUSH1 0xff",
                "600456605b60ff",
                vec![],
                false,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn jumpif() {
        let setups = vec![
            TestSetup::new(
                "PUSH1 0\nPUSH1 7\nJUMPI\nPUSH1 1\nJUMPDEST\nPUSH1 2\nPOP",
                "600060075760015b600250",
                vec!["0x1"],
                true,
            ),
            TestSetup::new(
                "PUSH1 1\nPUSH1 7\nJUMPI\nPUSH1 1\nJUMPDEST\nPUSH1 2",
                "600160075760015b6002",
                vec!["0x2"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }
}
