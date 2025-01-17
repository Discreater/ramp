// Copyright 2015 The Ramp Developers
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.

use crate::ll;
use crate::ll::limb::Limb;
use crate::mem;

use ll::limb_ptr::{Limbs, LimbsMut};

/// w <- a^b mod n:
/// - `wp`: the result pointer.
/// - `r_limbs`: the limbs count of `r`.
/// - `n`: modulus.
/// - `nquote0`: 1 / (r - n), because r = 1 << (n.limbs * Limbs::BITS), so (r - n) < Limb::MAX
/// - `a`: base.
/// - `bp`: exp
/// - `bn`: bit size of exp
/// 
/// # Safety
/// 
/// - `wp` must not overlapping with `n`, `a`, `bp` in `r_limbs` limbs.
/// - require `n` is odd.
/// - require `r_limbs` is greater than or equal to 0.
pub unsafe fn modpow(
    wp: LimbsMut,
    r_limbs: i32,
    n: Limbs,
    nquote0: Limb,
    a: Limbs,
    bp: Limbs,
    bn: i32,
) {
    let k = 6;

    let mut tmp = mem::TmpAllocator::new();
    let t = tmp.allocate((2 * r_limbs + 1) as usize);
    let scratch_mul = tmp.allocate(2 * r_limbs as usize);

    // base ^ 0..2^(k-1)
    let mut table = Vec::with_capacity(1 << k);
    let mut pow_0 = tmp.allocate(r_limbs as usize);
    *pow_0 = Limb(1);
    let pow_1 = tmp.allocate(r_limbs as usize);
    ll::copy_incr(a, pow_1, r_limbs);
    table.push(pow_0);
    table.push(pow_1);
    for _ in 2..(1 << k) {
        let next = tmp.allocate(r_limbs as usize);
        {
            let previous = table.last().unwrap();
            mul(
                next,
                r_limbs,
                pow_1.as_const(),
                previous.as_const(),
                n,
                nquote0,
                t,
                scratch_mul,
            );
        }
        table.push(next);
    }

    let exp_bit_length = ll::base::num_base_digits(bp, bn, 2);
    let block_count = (exp_bit_length + k - 1) / k;
    // recursive Wk = W(k)
    for i in (0..block_count).rev() {
        let mut block_value: usize = 0; // store the value of this block
        for j in 0..k {
            let p = i * k + j; // the pth bit
            if p < exp_bit_length
                && (*(bp.offset((p / Limb::BITS) as isize)) >> (p % Limb::BITS)) & Limb(1)
                    == Limb(1)
            {
                // If the pth bit is 1
                block_value |= 1 << j;
            }
        }
        // w^(2^k) = w^64
        for _ in 0..k {
            sqr(wp, r_limbs, wp.as_const(), n, nquote0, t, scratch_mul);
        }
        if block_value != 0 {
            mul(
                wp,
                r_limbs,
                wp.as_const(),
                table[block_value].as_const(),
                n,
                nquote0,
                t,
                scratch_mul,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[inline]
unsafe fn mul(
    wp: LimbsMut,
    r_limbs: i32,
    a: Limbs,
    b: Limbs,
    n: Limbs,
    nquote0: Limb,
    t: LimbsMut,
    scratch_mul: LimbsMut,
) {
    ll::mul::mul_rec(t, a, r_limbs, b, r_limbs, scratch_mul);
    redc(wp, r_limbs, n, nquote0, t)
}

#[inline]
/// Mgty square: a^2 * R^-1 mod n
unsafe fn sqr(
    wp: LimbsMut,
    r_limbs: i32,
    a: Limbs,
    n: Limbs,
    nquote0: Limb,
    t: LimbsMut,
    scratch_mul: LimbsMut,
) {
    ll::mul::sqr_rec(t, a, r_limbs, scratch_mul);
    redc(wp, r_limbs, n, nquote0, t)
}

/// # Safety
///
/// - `t` will be modified.
/// - `t` must have enough space to store `2 * r_limbs` limbs.
/// - `wp` must have enough space to store `r_limbs` limbs.
/// - require `r_limbs` > 0.
/// - `t` must not overlap with `n`.
/// - `wp` must not overlap with `n` and `t`, unless `wp` equal to one of `n` and `t.offset(r_limbs)`.
#[inline]
pub unsafe fn redc(wp: LimbsMut, r_limbs: i32, n: Limbs, nquote0: Limb, t: LimbsMut) {
    let mut carry = 0;
    for i in 0..r_limbs {
        carry = 0;
        let m = t.offset(i as _).0.wrapping_mul(nquote0.0 as _);
        for j in 0..r_limbs {
            let (h_mnj, l_mnj) = Limb(m).mul_hilo(*(n.offset(j as _)));
            let (s, c1) = t.offset((i + j) as _).add_overflow(l_mnj);
            let (s, c2) = s.add_overflow(Limb(carry));
            carry = c1 as ll::limb::BaseInt + c2 as ll::limb::BaseInt + h_mnj.0;
            *t.offset((i + j) as _) = s;
        }
        for j in (i + r_limbs)..(2 * r_limbs) {
            let (s, c) = t.offset(j as _).add_overflow(Limb(carry));
            carry = c as _;
            *t.offset(j as _) = s;
        }
    }
    if carry > 0
        || ll::cmp(t.offset(r_limbs as isize).as_const(), n, r_limbs) != ::std::cmp::Ordering::Less
    {
        ll::addsub::sub_n(wp, t.offset(r_limbs as isize).as_const(), n, r_limbs);
    } else {
        ll::copy_incr(t.offset(r_limbs as isize).as_const(), wp, r_limbs);
    }
}

pub fn inv1(x: Limb) -> Limb {
    let Limb(x) = x;
    let mut y = 1;
    for i in 2..(Limb::BITS) {
        if 1 << (i - 1) < (x.wrapping_mul(y) % (1 << i)) {
            y += 1 << (i - 1);
        }
    }
    if 1 << (Limb::BITS - 1) < x.wrapping_mul(y) {
        y += 1 << (Limb::BITS - 1);
    }
    Limb(y as _)
}

#[test]
fn test_inv1() {
    assert_eq!(inv1(Limb(23)).0.wrapping_mul(23), 1);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn test_inv1_64() {
    assert_eq!(
        inv1(Limb(193514046488575)).0.wrapping_mul(193514046488575),
        1
    );
}
