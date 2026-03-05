use std::slice;

#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn rust_fibonacci(n: i32) -> i64 {
    if n <= 1 {
        return n as i64;
    }
    let mut a: i64 = 0;
    let mut b: i64 = 1;
    for _ in 2..=n {
        let tmp = a + b;
        a = b;
        b = tmp;
    }
    b
}

#[no_mangle]
pub extern "C" fn rust_factorial(n: u32) -> u64 {
    (1..=n as u64).product()
}

#[no_mangle]
pub extern "C" fn rust_sort(ptr: *mut i32, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    let arr = unsafe { slice::from_raw_parts_mut(ptr, len) };
    arr.sort();
}

#[no_mangle]
pub extern "C" fn rust_sum(ptr: *const i32, len: usize) -> i64 {
    if ptr.is_null() || len == 0 {
        return 0;
    }
    let arr = unsafe { slice::from_raw_parts(ptr, len) };
    arr.iter().map(|&x| x as i64).sum()
}

#[no_mangle]
pub extern "C" fn rust_fnv_hash(ptr: *const u8, len: usize) -> u32 {
    if ptr.is_null() || len == 0 {
        return 0;
    }
    let data = unsafe { slice::from_raw_parts(ptr, len) };
    let mut hash: u32 = 0x811c9dc5;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

#[no_mangle]
pub extern "C" fn rust_gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

#[no_mangle]
pub extern "C" fn rust_is_prime(n: u64) -> i32 {
    if n < 2 {
        return 0;
    }
    if n < 4 {
        return 1;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return 0;
    }
    let mut i = 5u64;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return 0;
        }
        i += 6;
    }
    1
}
