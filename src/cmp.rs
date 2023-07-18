use crate::utils::Stack;
use primitive_types::U256;

const LT: u8 = 0x10;
const GT: u8 = 0x11;
const SLT: u8 = 0x12;
const SGT: u8 = 0x13;
const EQ: u8 = 0x14;
const ISZERO: u8 = 0x15;
const AND: u8 = 0x16;
const OR: u8 = 0x17;
const XOR: u8 = 0x18;
const NOT: u8 = 0x19;
const BYTE: u8 = 0x1a;
const SHL: u8 = 0x1b;
const SHR: u8 = 0x1c;
const SAR: u8 = 0x1d;

pub fn exec(opcode: u8, stack: &mut Vec<U256>) {
    match opcode {
        LT => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            if left < right {
                stack.push(U256::one());
            } else {
                stack.push(U256::zero());
            }
        }
        GT => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            if left > right {
                stack.push(U256::one());
            } else {
                stack.push(U256::zero());
            }
        }
        SLT => todo!(),
        SGT => todo!(),
        EQ => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            if left == right {
                stack.push(U256::one());
            } else {
                stack.push(U256::zero());
            }
        }
        ISZERO => {
            let left = stack.safe_pop();
            if left == U256::zero() {
                stack.push(U256::one());
            } else {
                stack.push(U256::zero());
            }
        }
        AND => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            stack.push(left & right);
        }
        OR => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            stack.push(left | right);
        }
        XOR => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            stack.push(left ^ right);
        }
        NOT => {
            let left = stack.safe_pop();
            stack.push(!left);
        }
        BYTE => {
            let offset = stack.safe_pop();
            let value = stack.safe_pop();

            let offset = offset.as_usize();

            if offset > 31 || value == U256::zero() {
                stack.push(U256::zero());
            } else {
                let byte = value.byte(31 - offset);
                stack.push(U256::from(byte));
            }
        }
        SHL => {
            let right = stack.safe_pop();
            let left = stack.safe_pop();
            if right >= U256::from(256) || left == U256::zero() {
                stack.push(U256::zero());
            } else {
                let right: u64 = right.as_u64();
                let shifted = left << right as usize;
                stack.push(shifted);
            }
        }
        SHR => {
            let right = stack.safe_pop();
            let left = stack.safe_pop();
            if right >= U256::from(256) || left == U256::zero() {
                stack.push(U256::zero());
            } else {
                let right: u64 = right.as_u64();
                let shifted = left >> right as usize;
                stack.push(shifted);
            }
        }
        SAR => todo!(),
        _ => panic!("Not a cmp or bitwise opcode!"),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{run_test, TestSetup};

    #[test]
    fn less_than() {
        let setups = vec![
            TestSetup::new("PUSH1 10\nPUSH1 9\nLT", "600a600910", vec!["0x1"], true),
            TestSetup::new("PUSH1 9\nPUSH1 10\nLT", "6009600a10", vec!["0x0"], true),
            TestSetup::new("PUSH1 9\nPUSH1 9\nLT", "6009600910", vec!["0x0"], true),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn greater_than() {
        let setups = vec![
            TestSetup::new("PUSH1 10\nPUSH1 9\nGT", "600a600911", vec!["0x0"], true),
            TestSetup::new("PUSH1 9\nPUSH1 10\nGT", "6009600a11", vec!["0x1"], true),
            TestSetup::new("PUSH1 9\nPUSH1 9\nGT", "6009600911", vec!["0x0"], true),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn equals() {
        let setups = vec![
            TestSetup::new("PUSH1 10\nPUSH1 9\nGT", "600a600914", vec!["0x0"], true),
            TestSetup::new("PUSH1 9\nPUSH1 10\nGT", "6009600a14", vec!["0x0"], true),
            TestSetup::new("PUSH1 9\nPUSH1 9\nGT", "6009600914", vec!["0x1"], true),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn is_zero() {
        let setups = vec![
            TestSetup::new("PUSH1 10\nISZERO", "600a15", vec!["0x0"], true),
            TestSetup::new("PUSH1 0\nISZERO", "600015", vec!["0x1"], true),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn not() {
        let setups = vec![TestSetup::new(
            "PUSH1 0x0f\nNOT",
            "600f19",
            vec!["0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0"],
            true,
        )];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn and() {
        let setups = vec![TestSetup::new(
            "PUSH1 0x0e\nPUSH1 0x03\nAND",
            "600e600316",
            vec!["0x2"],
            true,
        )];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn or() {
        let setups = vec![TestSetup::new(
            "PUSH1 0x0e\nPUSH1 0x03\nOR",
            "600e600317",
            vec!["0xf"],
            true,
        )];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn xor() {
        let setups = vec![TestSetup::new(
            "PUSH1 0xf0\nPUSH1 0x0f\nXOR",
            "600f60f018",
            vec!["0xff"],
            true,
        )];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn left_shift() {
        let setups = vec![
            TestSetup::new( // 1 << 2 = 4
                "PUSH1 0x1\nPUSH1 0x2\nSHL", 
                "600160021b", 
                vec!["0x4"], 
                true
            ),
            TestSetup::new( // 0xff00... << 4 = 0xf000...
                "PUSH32 0xFF00000000000000000000000000000000000000000000000000000000000000\nPUSH1 4\nSHL", 
                "7fff0000000000000000000000000000000000000000000000000000000000000060041b", 
                vec!["0xf000000000000000000000000000000000000000000000000000000000000000"], 
                true
            ),
            TestSetup::new( // 0xMAX256 << 255 = 0x8000...
                "PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\nPUSH1 255\nSHL", 
                "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60ff1b", 
                vec!["0x8000000000000000000000000000000000000000000000000000000000000000"], 
                true
            ),
            TestSetup::new( // 0xMAX256 << 256 = 0x0
                "PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\nPUSH2 256\nSHL", 
                "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff6101001b", 
                vec!["0x0"], 
                true
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn right_shift() {
        let setups = vec![
            TestSetup::new( // 4 << 2 = 1
                "PUSH1 0x4\nPUSH1 0x2\nSHR",
                "600460021c", 
                vec!["0x1"], 
                true
            ),
            TestSetup::new( // 0xffff >> 0x8 = 0xff
                "PUSH2 0xffff\nPUSH1 0x8\nSHR",
                "61ffff60081c", 
                vec!["0xff"], 
                true
            ),
            TestSetup::new( // 0xMAX256 >> 255 = 1
                "PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\nPUSH1 0xff\nSHR",
                "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60ff1c", 
                vec!["0x1"], 
                true
            ),
            TestSetup::new( // 0xMAX256 >> 256 = 0
                "PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\nPUSH2 0x0100\nSHR",
                "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff6101001c", 
                vec!["0x0"], 
                true
            ),
            TestSetup::new( // 4 >> 0xffff = 0
                "PUSH1 0x4\nPUSH2 0xffff\nSHR",
                "600461ffff1c", 
                vec!["0x0"], 
                true
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn byte() {
        let setups = vec![
            TestSetup::new(
                // 0x00...ff[31] = 0xff
                "PUSH1 0xff\nPUSH1 31\nBYTE",
                "60ff601f1a",
                vec!["0xff"],
                true,
            ),
            TestSetup::new(
                // 0xff...ff[32] = 0
                "PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\nPUSH1 32\nBYTE",
                "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60201a",
                vec!["0x0"],
                true,
            ),
            TestSetup::new(
                // 0xff11...ff[1] = 0x11
                "PUSH32 0xff11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\nPUSH1 1\nBYTE",
                "7fff11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60011a",
                vec!["0x11"],
                true,
            ),
            TestSetup::new(
                // 0x11ff...ff[0] = 0x11
                "PUSH32 0x11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\nPUSH1 0\nBYTE",
                "7f11ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff60001a",
                vec!["0x11"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }
}
