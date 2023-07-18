use primitive_types::U256;

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

    struct TestSetup {
        asm: String,
        bin: String,
        expect_stack: Vec<String>,
        expect_success: bool,
    }

    impl TestSetup {
        fn new(asm: &str, bin: &str, expect_stack: Vec<&str>, expect_success: bool) -> Self {
            let stack = expect_stack.iter().map(|s| s.to_string()).collect();
            Self {
                asm: asm.to_string(),
                bin: bin.to_string(),
                expect_stack: stack,
                expect_success,
            }
        }
    }

    fn run_test(setup: TestSetup) {
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
        let asm = "PUSH1 1\nSTOP\nPUSH1 2";
        let bin = "6001006002";
        let expect_stack = vec!["0x1"];
        let expect_success = true;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn add_2plus2() {
        let asm = "PUSH1 0x02\nPUSH1 0x02\nADD";
        let bin = "6002600201";
        let expect_stack = vec!["0x4"];
        let expect_success = true;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    #[should_panic]
    fn add_2plus2is5() {
        let asm = "PUSH1 0x02\nPUSH1 0x02\nADD";
        let bin = "6002600201";
        let expect_stack = vec!["0x5"];
        let expect_success = true;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn add_overflow() {
        let asm = "PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\nPUSH1 0x2\nADD";
        let bin = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff600201";
        let expect_stack = vec!["0x1"];
        let expect_success = true;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn mul() {
        let setups = vec![
            TestSetup::new(
                // 2 * 3 = 6
                "PUSH1 0x02\nPUSH1 0x03\nMUL",
                "6002600302",
                vec!["0x6"],
                true,
            ),
            TestSetup::new(
                // 0 * 3 = 0
                "PUSH1 0x00\nPUSH1 0x03\nMUL",
                "6000600302",
                vec!["0x0"],
                true,
            ),
            TestSetup::new(
                // 1 * 3 = 3
                "PUSH1 0x01\nPUSH1 0x03\nMUL",
                "6001600302",
                vec!["0x3"],
                true,
            ),
            TestSetup::new(
                // 1 * 1 = 1
                "PUSH1 0x01\nPUSH1 0x01\nMUL",
                "6001600102",
                vec!["0x1"],
                true,
            ),
            TestSetup::new(
                // 0 * 0 = 0
                "PUSH1 0x00\nPUSH1 0x00\nMUL",
                "6000600002",
                vec!["0x0"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    #[should_panic]
    fn mul_bad() {
        let asm = "PUSH1 0x02\nPUSH1 0x03\nMUL";
        let bin = "6002600302";
        let expect_stack = vec!["0x7"];
        let expect_success = true;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn mul_overflow() {
        let asm = "PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\nPUSH1 0x2\nMUL";
        let bin = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff600202";
        let expect_stack =
            vec!["0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"];
        let expect_success = true;

        let setup = TestSetup::new(asm, bin, expect_stack, expect_success);

        run_test(setup);
    }

    #[test]
    fn sub() {
        let setups = vec![
            TestSetup::new(
                // 3 - 2 = 1
                "PUSH1 0x02\nPUSH1 0x03\nSUB",
                "6002600303",
                vec!["0x1"],
                true,
            ),
            TestSetup::new(
                // 3 - 0 = 3
                "PUSH1 0x00\nPUSH1 0x03\nSUB",
                "6000600303",
                vec!["0x3"],
                true,
            ),
            TestSetup::new(
                // 0 - 0 = 0
                "PUSH1 0x00\nPUSH1 0x00\nSUB",
                "6000600003",
                vec!["0x0"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn sub_underflow() {
        let setup = TestSetup::new(
            // 2 - 3 = MAX 
            "PUSH1 0x03\nPUSH1 0x02\nSUB",
            "6003600203",
            vec!["0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
            true,
        );

        run_test(setup);
    }

    #[test]
    fn div() {
        let setups = vec![
            TestSetup::new(
                // 4 / 2 = 1
                "PUSH1 0x02\nPUSH1 0x04\nDIV",
                "6002600404",
                vec!["0x2"],
                true,
            ),
            TestSetup::new(
                // 3 / 2 = 1
                "PUSH1 0x02\nPUSH1 0x03\nDIV",
                "6002600304",
                vec!["0x1"],
                true,
            ),
            TestSetup::new(
                // 2 / 3 = 0
                "PUSH1 0x03\nPUSH1 0x02\nDIV",
                "6003600204",
                vec!["0x0"],
                true,
            ),
            TestSetup::new(
                // 3 / 0 = 0
                "PUSH1 0x00\nPUSH1 0x03\nDIV",
                "6000600304",
                vec!["0x0"],
                true,
            ),
            TestSetup::new(
                // 0 / 3 = 0
                "PUSH1 0x03\nPUSH1 0x00\nDIV",
                "6003600004",
                vec!["0x0"],
                true,
            ),
            TestSetup::new(
                // 0 / 0 = 0
                "PUSH1 0x00\nPUSH1 0x00\nDIV",
                "6000600004",
                vec!["0x0"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }
}
