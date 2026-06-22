// Runtime field arithmetic for circom-generated Rust witness calculators.
// All values are malachite Integer in the range [0, prime).

use malachite::integer::Integer;
use malachite::natural::Natural;
use malachite::num::arithmetic::traits::ModPow;
use malachite::num::conversion::traits::{ExactFrom, SaturatingFrom};

pub type FrElement = Integer;

#[inline]
fn modp(v: Integer, prime: &Integer) -> Integer {
    v % prime
}

pub fn fr_add(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    modp(a + b, prime)
}

pub fn fr_sub(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    if a >= b {
        a - b
    } else {
        let mut r = prime.clone();
        r += a;
        r -= b;
        r
    }
}

pub fn fr_mul(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    modp(a * b, prime)
}

pub fn fr_neg(a: &FrElement, prime: &Integer) -> FrElement {
    if *a == Integer::from(0u32) {
        Integer::from(0u32)
    } else {
        prime - a
    }
}

pub fn fr_inv(a: &FrElement, prime: &Integer) -> FrElement {
    if *a == Integer::from(0u32) {
        panic!("fr_inv: division by zero");
    }
    // Extended Euclidean Algorithm
    let mut r0 = prime.clone();
    let mut r1 = a.clone();
    let mut t0 = Integer::from(0u32);
    let mut t1 = Integer::from(1u32);
    while r1 != Integer::from(0u32) {
        let q = &r0 / &r1;
        let r2 = &r0 - &q * &r1;
        let t2 = &t0 - &q * &t1;
        r0 = r1;
        r1 = r2;
        t0 = t1;
        t1 = t2;
    }
    if t0 < Integer::from(0u32) {
        t0 += prime;
    }
    t0
}

pub fn fr_div(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    fr_mul(a, &fr_inv(b, prime), prime)
}

pub fn fr_pow(base: &FrElement, exp: &FrElement, prime: &Integer) -> FrElement {
    let base_n = Natural::try_from(base.clone()).expect("fr_pow: base must be non-negative");
    let exp_n  = Natural::try_from(exp.clone()).expect("fr_pow: exp must be non-negative");
    let prime_n = Natural::try_from(prime.clone()).expect("fr_pow: prime must be positive");
    Integer::from(base_n.mod_pow(exp_n, &prime_n))
}

pub fn fr_idiv(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    let a_int = to_signed(a, prime);
    let b_int = to_signed(b, prime);
    from_signed(div_floor(&a_int, &b_int), prime)
}

pub fn fr_mod(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    let a_int = to_signed(a, prime);
    let b_int = to_signed(b, prime);
    from_signed(mod_floor(&a_int, &b_int), prime)
}

pub fn fr_shl(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    let shift = u64::saturating_from(b);
    modp(a << shift, prime)
}

pub fn fr_shr(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    let a_int = to_signed(a, prime);
    let shift = u64::saturating_from(b);
    from_signed(a_int >> shift, prime)
}

pub fn fr_lt(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    if to_signed(a, prime) < to_signed(b, prime) { Integer::from(1u32) } else { Integer::from(0u32) }
}

pub fn fr_gt(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    if to_signed(a, prime) > to_signed(b, prime) { Integer::from(1u32) } else { Integer::from(0u32) }
}

pub fn fr_leq(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    if to_signed(a, prime) <= to_signed(b, prime) { Integer::from(1u32) } else { Integer::from(0u32) }
}

pub fn fr_geq(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    if to_signed(a, prime) >= to_signed(b, prime) { Integer::from(1u32) } else { Integer::from(0u32) }
}

pub fn fr_eq(a: &FrElement, b: &FrElement, _prime: &Integer) -> FrElement {
    if a == b { Integer::from(1u32) } else { Integer::from(0u32) }
}

pub fn fr_neq(a: &FrElement, b: &FrElement, _prime: &Integer) -> FrElement {
    if a != b { Integer::from(1u32) } else { Integer::from(0u32) }
}

pub fn fr_land(a: &FrElement, b: &FrElement, _prime: &Integer) -> FrElement {
    if *a != Integer::from(0u32) && *b != Integer::from(0u32) { Integer::from(1u32) } else { Integer::from(0u32) }
}

pub fn fr_lor(a: &FrElement, b: &FrElement, _prime: &Integer) -> FrElement {
    if *a != Integer::from(0u32) || *b != Integer::from(0u32) { Integer::from(1u32) } else { Integer::from(0u32) }
}

pub fn fr_lnot(a: &FrElement, _prime: &Integer) -> FrElement {
    if *a == Integer::from(0u32) { Integer::from(1u32) } else { Integer::from(0u32) }
}

pub fn fr_band(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    modp(a & b, prime)
}

pub fn fr_bor(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    modp(a | b, prime)
}

pub fn fr_bxor(a: &FrElement, b: &FrElement, prime: &Integer) -> FrElement {
    modp(a ^ b, prime)
}

pub fn fr_bnot(a: &FrElement, prime: &Integer) -> FrElement {
    let mask = prime - Integer::from(1u32);
    modp(&mask ^ a, prime)
}

pub fn fr_to_int(a: &FrElement, prime: &Integer) -> usize {
    let v = to_signed(a, prime);
    assert!(v >= Integer::from(0u32), "fr_to_int: negative value used as index");
    u64::exact_from(&v) as usize
}

pub fn fr_is_true(a: &FrElement) -> bool {
    *a != Integer::from(0u32)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn to_signed(a: &FrElement, prime: &Integer) -> Integer {
    let half = prime >> 1u64;
    if a > &half {
        a - prime
    } else {
        a.clone()
    }
}

fn from_signed(v: Integer, prime: &Integer) -> FrElement {
    let mut r = v % prime;
    if r < Integer::from(0u32) {
        r += prime;
    }
    r
}

fn div_floor(a: &Integer, b: &Integer) -> Integer {
    let q = a / b;
    let r = a % b;
    let zero = Integer::from(0u32);
    if r != zero && (r < zero) != (*b < zero) {
        q - Integer::from(1u32)
    } else {
        q
    }
}

fn mod_floor(a: &Integer, b: &Integer) -> Integer {
    let r = a % b;
    let zero = Integer::from(0u32);
    if r != zero && (r < zero) != (*b < zero) {
        r + b
    } else {
        r
    }
}
