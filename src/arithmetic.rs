use crate::utils::Stack;
use primitive_types::{U256, U512};

// arithmetic opcodes
const ADD: u8 = 0x01;
const MUL: u8 = 0x02;
const SUB: u8 = 0x03;
const DIV: u8 = 0x04;
const SDIV: u8 = 0x05;
const MOD: u8 = 0x06;
const SMOD: u8 = 0x07;
const ADDMOD: u8 = 0x08;
const MULMOD: u8 = 0x09;
const EXP: u8 = 0x0a;
const SIGNEXTEND: u8 = 0x0b;

fn smod(mut left: U256, mut right: U256) -> U256 {
    if right == U256::zero() {
        return right;
    }

    let left_neg = is_negative(left);
    let right_neg = is_negative(right);

    // both are positive, return early
    if !left_neg && !right_neg {
        return left.checked_rem(right).unwrap_or(U256::zero());
    }

    // convert to positive representation if either value is negative
    if left_neg {
        left = inv(left);
        println!("left {}", left);
    }
    if right_neg {
        right = inv(right);
    }

    // postiive result
    let pos = left.checked_rem(right).unwrap_or(U256::zero());

    // if signs are same, invert and then return
    if left_neg && right_neg {
        return inv(pos);
    }
    pos
}

fn sdiv(mut left: U256, mut right: U256) -> U256 {
    if right == U256::zero() {
        return right;
    }

    let left_neg = is_negative(left);
    let right_neg = is_negative(right);

    // both are positive, return early
    if !left_neg && !right_neg {
        return left.checked_div(right).unwrap_or(U256::zero());
    }

    // convert to positive representation if either value is negative
    if left_neg {
        left = inv(left);
    }
    if right_neg {
        right = inv(right);
    }

    // postiive result
    let pos = left.checked_div(right).unwrap_or(U256::zero());

    // if signs are different, invert and then return
    if left_neg ^ right_neg {
        return inv(pos);
    }
    pos
}

// invert a signed U256
fn inv(val: U256) -> U256 {
    !val + 1
}

// check if the given number is negative
fn is_negative(x: U256) -> bool {
    x.bit(31)
}

fn sign_extend(stack: &mut Vec<U256>) {
    // number of bytes to read
    let b = stack.safe_pop().as_usize();

    // value to extend
    let x = stack.safe_pop();
    let mut bytes = [0; 32];
    x.to_little_endian(&mut bytes);
    println!("{:x?}", bytes);

    // get sign bit
    let sign = (bytes[b] & 0b1000_0000) > 0;
    println!("{}", sign);

    // if it's one, pad ones, else pad zeros
    if sign {
        for (i, byte) in bytes.iter_mut().enumerate() {
            if i > b {
                *byte = 0b1111_1111;
            }
        }
    } else {
        for (i, byte) in bytes.iter_mut().enumerate() {
            if i > b {
                *byte = 0;
            }
        }
    }

    let res = U256::from_little_endian(&bytes);
    stack.push(res);
}

pub fn exec(opcode: u8, stack: &mut Vec<U256>) {
    match opcode {
        ADD => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            let (res, _) = left.overflowing_add(right);
            stack.push(res);
        }
        MUL => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            let (res, _) = left.overflowing_mul(right);
            stack.push(res);
        }
        SUB => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            let (res, _) = left.overflowing_sub(right);
            stack.push(res);
        }
        DIV => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            let res = left.checked_div(right).unwrap_or(U256::zero());
            stack.push(res);
        }
        SDIV => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();

            stack.push(sdiv(left, right));
        }
        MOD => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            let res = left.checked_rem(right).unwrap_or(U256::zero());
            stack.push(res);
        }
        SMOD => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            let res = smod(left, right);
            stack.push(res);
        }
        ADDMOD => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            let div = stack.pop().unwrap();
            let res = left
                .overflowing_add(right)
                .0
                .checked_rem(div)
                .unwrap_or(U256::zero());
            stack.push(res)
        }
        MULMOD => {
            let left = stack.safe_pop();
            let right = stack.safe_pop();
            let div = U512::from(stack.pop().unwrap());
            let res = left.full_mul(right).checked_rem(div).unwrap();

            let mut bytes = [0; 64];
            res.to_little_endian(&mut bytes);

            let res = U256::from_little_endian(&bytes[0..32]);
            stack.push(res)
        }
        EXP => {
            let base = stack.safe_pop();
            let pow = stack.safe_pop();
            let res = base.overflowing_pow(pow).0;
            stack.push(res);
        }
        SIGNEXTEND => {
            sign_extend(stack);
        }
        _ => panic!("Unrecognized arithmetic opcode"),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{run_test, TestSetup};

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

    #[test]
    fn modulo() {
        let setups = vec![
            TestSetup::new(
                // 10 % 3 = 1
                "PUSH1 03\nPUSH1 10\nMOD",
                "6003600a06",
                vec!["0x1"],
                true,
            ),
            TestSetup::new(
                // 5 % 17 = 5
                "PUSH1 17\nPUSH1 5\nMOD",
                "6011600506",
                vec!["0x5"],
                true,
            ),
            TestSetup::new(
                // 3 % 0 = 0
                "PUSH1 0x00\nPUSH1 0x03\nMOD",
                "6000600306",
                vec!["0x0"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn add_mod() {
        let setups = vec![
            TestSetup::new(
                // 10 + 10 mod 8 = 4
                "PUSH1 8\nPUSH1 10\nPUSH1 10\nADDMOD",
                "6008600a600a08",
                vec!["0x04"],
                true,
            ),
            TestSetup::new(
                // wrapped
                "PUSH1 2\nPUSH1 2\nPUSH32 MAX\nADDMOD",
                "600260027fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff08",
                vec!["0x01"],
                true,
            ),
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn mul_mod() {
        let setups = vec![
             TestSetup::new(
                 // 10 * 10 mod 8 = 4
                 "PUSH1 8\nPUSH1 10\nPUSH1 10\nMULMOD",
                 "6008600a600a09",
                 vec!["0x04"],
                 true,
             ),
             TestSetup::new(
                 // wrapped
                 "PUSH1 12\nPUSH32 MAX\nPUSH32 MAX\nMULMOD",
                 "600c7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff09",
                 vec!["0x09"],
                 true,
             ),
         ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn exp() {
        let setups = vec![TestSetup::new(
            "PUSH1 2\nPUSH1 10\nEXP",
            "6002600a0a",
            vec!["0x64"],
            true,
        )];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn sign_extend_positive() {
        let setups = vec![TestSetup::new(
            "PUSH1 0xff\nPUSH1 0\nSIGNEXTEND",
            "60ff60000b",
            vec!["0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"],
            true,
        )];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn sdiv() {
        let setups = vec![
            TestSetup::new("PUSH1 10\nPUSH1 10\nSDIV", "600a600a05", vec!["0x1"], true),
            TestSetup::new(
                "PUSH32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\nPUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe\nSDIV",
                "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe05",
                vec!["0x2"],
                true,
            ),
            TestSetup::new(
                "PUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe\nPUSH1 10\nSDIV",
                "7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe600a05",
                vec!["0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffb"],
                true
            ),
            TestSetup::new(
                "PUSH1 0x00\nPUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd\nSDIV",
                "60007ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd05",
                vec!["0x0"],
                true,
            )
        ];

        for setup in setups {
            run_test(setup);
        }
    }

    #[test]
    fn smod() {
        let setups = vec![
            TestSetup::new("PUSH1 3\nPUSH1 10\nSMOD", "6003600a07", vec!["0x1"], true),
            TestSetup::new(
                "PUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd\nPUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8\nSMOD",
                "7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff807",
                vec!["0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"],
                true,
            ),
            TestSetup::new(
                "PUSH1 0x00\nPUSH32 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd\nSMOD",
                "60007ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd07",
                vec!["0x0"],
                true,
            )
        ];

        for setup in setups {
            run_test(setup);
        }
    }
}
