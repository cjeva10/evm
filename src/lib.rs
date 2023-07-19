use crate::utils::ValidJumps;
use primitive_types::U256;
use std::collections::HashMap;

mod arithmetic;
mod cmp;
mod dup_swap;
mod flow;
mod hash;
mod memory;
mod utils;

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
const INVALID: u8 = 0xfe;
const PC: u8 = 0x58;

struct Program<'a> {
    code: &'a [u8],
    pc: usize,
    state: ProgramState,
}

impl<'a> Program<'a> {
    fn new<A: AsRef<[u8]> + 'a>(_code: &'a A) -> Self {
        let code = _code.as_ref();
        Self {
            code,
            pc: 0,
            state: ProgramState::new(),
        }
    }
}

struct ProgramState {
    stack: Vec<U256>,
    memory: Memory,
}

pub struct Memory {
    pub data: HashMap<U256, u8>,
    pub size: U256,
}

impl ProgramState {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            memory: Memory {
                data: HashMap::new(),
                size: U256::zero(),
            },
        }
    }
}

pub fn evm(_code: impl AsRef<[u8]>) -> EvmResult {
    let mut program = Program::new(&_code);

    // get all the valid jump destinations up front
    let jumps = ValidJumps::new(program.code).jumps;

    while program.pc < program.code.len() {
        let opcode = program.code[program.pc];
        program.pc += 1;

        // push byte value onto the stack
        if opcode >= PUSH1 && opcode <= PUSH32 {
            let size = (opcode - PUSH1 + 1) as usize;
            program.state.stack.push(U256::from_big_endian(
                &program.code[program.pc..program.pc + size],
            ));
            program.pc += size;
            continue;
        }

        // arithmetic operations
        if opcode >= 0x01 && opcode <= 0x0b {
            arithmetic::exec(opcode, &mut program.state.stack);
            continue;
        }

        // comparison operations
        if opcode >= 0x10 && opcode < 0x20 {
            cmp::exec(opcode, &mut program.state.stack);
            continue;
        }

        // dup and swap operations
        if opcode >= 0x80 && opcode <= 0x9f {
            dup_swap::exec(opcode, &mut program.state.stack);
            continue;
        }

        // control flow opcodes
        if opcode == 0x56 || opcode == 0x57 || opcode == 0x5b {
            let result = flow::exec(
                opcode,
                &mut program.state.stack,
                &mut program.pc,
                program.code,
                &jumps,
            );
            // control flow opcodes can terminate the program
            if let Some(result) = result {
                return result;
            }
            continue;
        }

        // memory opcodes
        if (opcode >= 0x51 && opcode <= 0x53) || opcode == 0x59 {
            let result = memory::exec(opcode, &mut program.state.stack, &mut program.state.memory);
            if let Some(result) = result {
                return result;
            }
            continue;
        }

        // hash opcodes
        if opcode == 0x20 {
            hash::exec(opcode, &mut program.state.stack, &mut program.state.memory);
        }

        // basic opcodes
        match opcode {
            STOP => {
                return EvmResult {
                    stack: program.state.stack,
                    success: true,
                }
            }
            PUSH0 => program.state.stack.push(U256::zero()),
            POP => {
                program.state.stack.pop();
            }
            PC => {
                program.state.stack.push(U256::from(program.pc - 1));
            }
            INVALID => {
                return EvmResult {
                    stack: program.state.stack,
                    success: false,
                }
            }
            _ => (),
        }
    }

    return EvmResult {
        stack: program.state.stack,
        success: true,
    };
}

#[cfg(test)]
mod tests {
    use crate::evm;
    use primitive_types::U256;

    pub struct TestSetup {
        asm: String,
        bin: String,
        expect_stack: Vec<String>,
        expect_success: bool,
    }

    impl TestSetup {
        pub fn new(asm: &str, bin: &str, expect_stack: Vec<&str>, expect_success: bool) -> Self {
            let stack = expect_stack.iter().map(|s| s.to_string()).collect();
            Self {
                asm: asm.to_string(),
                bin: bin.to_string(),
                expect_stack: stack,
                expect_success,
            }
        }
    }

    pub fn run_test(setup: TestSetup) {
        let asm = setup.asm;
        let bin = setup.bin;
        let expect_stack = setup.expect_stack;
        let expect_success = setup.expect_success;

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

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn push0() {
        let asm = "PUSH0";
        let bin = "5f";
        let expect_stack = vec!["0x0"];
        let expect_success = true;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn pushes_one_value() {
        let setups: Vec<TestSetup> = vec![
            TestSetup::new("PUSH1 0x1", "6001", vec!["0x1"], true),
            TestSetup::new("PUSH2 0x1122", "611122", vec!["0x1122"], true),
            TestSetup::new("PUSH4 0x11223344", "6311223344", vec!["0x11223344"], true),
            TestSetup::new(
                "PUSH6 0x112233445566",
                "65112233445566",
                vec!["0x112233445566"],
                true,
            ),
            TestSetup::new(
                "PUSH10 0x112233445566778899aa",
                "69112233445566778899aa",
                vec!["0x112233445566778899aa"],
                true,
            ),
            TestSetup::new(
                "PUSH11 0x112233445566778899aabb",
                "6a112233445566778899aabb",
                vec!["0x112233445566778899aabb"],
                true,
            ),
            TestSetup::new(
                "PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                vec!["0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn push_twice() {
        let setup = TestSetup::new("PUSH1 1\nPUSH1 2", "60016002", vec!["0x1", "0x2"], true);

        run_test(setup);
    }

    #[test]
    fn pop() {
        let asm = "PUSH1 1\nPUSH1 2\nPOP";
        let bin = "6001600250";
        let expect_stack = vec!["0x1"];
        let expect_success = true;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn stop_midway() {
        let asm = "PUSH1 1\nPUSH1 2\nSTOP\nPUSH1 2";
        let bin = "60016002006002";
        let expect_stack = vec!["0x1", "0x2"];
        let expect_success = true;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn invalid() {
        let asm = "INVALID";
        let bin = "fe";
        let expect_stack = vec![];
        let expect_success = false;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn pc() {
        let setups = vec![
            TestSetup::new("PC", "58", vec!["0x0"], true),
            TestSetup::new("PUSH1 0\nPOP\nPC", "60005058", vec!["0x3"], true),
        ];

        for setup in setups {
            run_test(setup);
        }
    }
}
