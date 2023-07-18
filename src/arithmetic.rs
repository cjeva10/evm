use primitive_types::{U256, U512};

const ADD: u8 = 0x01;
const MUL: u8 = 0x02;
const SUB: u8 = 0x03;
const DIV: u8 = 0x04;
const SDIV: u8 = 0x05;
const MOD: u8 = 0x06;
const ADDMOD: u8 = 0x08;
const MULMOD: u8 = 0x09;
const EXP: u8 = 0x0a;
const SIGNEXTEND: u8 = 0x0b;

fn sdiv(stack: &mut Vec<U256>) {
    let num = stack.pop().unwrap();
    let mut num_bytes = [0; 32];

    let den = stack.pop().unwrap();
    let mut den_bytes = [0; 32];

    num.to_little_endian(&mut num_bytes);
    den.to_little_endian(&mut den_bytes);

    let _num_sign = (num_bytes[31] & 0b1000_0000) > 0;
    let _den_sign = (den_bytes[31] & 0b1000_0000) > 0;

    num_bytes[31] &= 0b0111_1111; // make sure the sign bit is zero
    den_bytes[31] &= 0b0111_1111;
}

fn sign_extend(stack: &mut Vec<U256>) {
    // number of bytes to read
    let b = stack.pop().unwrap().as_usize();

    // value to extend
    let x = stack.pop().unwrap();
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

pub fn do_arithmetic(opcode: u8, stack: &mut Vec<U256>) {
    match opcode {
        ADD => {
            let left = stack.pop().unwrap();
            let right = stack.pop().unwrap();
            let (res, _) = left.overflowing_add(right);
            stack.push(res);
        }
        MUL => {
            let left = stack.pop().unwrap();
            let right = stack.pop().unwrap();
            let (res, _) = left.overflowing_mul(right);
            stack.push(res);
        }
        SUB => {
            let left = stack.pop().unwrap();
            let right = stack.pop().unwrap();
            let (res, _) = left.overflowing_sub(right);
            stack.push(res);
        }
        DIV => {
            let left = stack.pop().unwrap();
            let right = stack.pop().unwrap();
            let res = left.checked_div(right).unwrap_or(U256::zero());
            stack.push(res);
        }
        MOD => {
            let left = stack.pop().unwrap();
            let right = stack.pop().unwrap();
            let res = left.checked_rem(right).unwrap_or(U256::zero());
            stack.push(res);
        }
        ADDMOD => {
            let left = stack.pop().unwrap();
            let right = stack.pop().unwrap();
            let div = stack.pop().unwrap();
            let res = left
                .overflowing_add(right)
                .0
                .checked_rem(div)
                .unwrap_or(U256::zero());
            stack.push(res)
        }
        MULMOD => {
            let left = stack.pop().unwrap();
            let right = stack.pop().unwrap();
            let div = U512::from(stack.pop().unwrap());
            let res = left.full_mul(right).checked_rem(div).unwrap();

            let mut bytes = [0; 64];
            res.to_little_endian(&mut bytes);

            let res = U256::from_little_endian(&bytes[0..32]);
            stack.push(res)
        }
        EXP => {
            let base = stack.pop().unwrap();
            let pow = stack.pop().unwrap();
            let res = base.overflowing_pow(pow).0;
            stack.push(res);
        }
        SIGNEXTEND => {
            sign_extend(stack);
        }
        SDIV => {
            sdiv(stack);
        }
        _ => panic!("Unrecognized arithmetic opcode"),
    }
}

