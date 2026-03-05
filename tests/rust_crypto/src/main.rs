// Rust test program: crypto-like operations, iterators, closures, generics
// Tests obfuscator against Rust's code generation patterns (monomorphization,
// unwinding tables, enum layout, etc.)

use std::process::ExitCode;

static mut PASS: i32 = 0;
static mut FAIL: i32 = 0;

macro_rules! check {
    ($cond:expr, $msg:expr) => {
        unsafe {
            if $cond {
                println!("  [OK] {}", $msg);
                PASS += 1;
            } else {
                println!("  [FAIL] {}", $msg);
                FAIL += 1;
            }
        }
    };
}

// --- TEA (Tiny Encryption Algorithm) ---
fn tea_encrypt(v: &mut [u32; 2], key: &[u32; 4]) {
    let (mut v0, mut v1) = (v[0], v[1]);
    let mut sum: u32 = 0;
    let delta: u32 = 0x9E3779B9;
    for _ in 0..32 {
        sum = sum.wrapping_add(delta);
        v0 = v0.wrapping_add(
            ((v1 << 4).wrapping_add(key[0])) ^ (v1.wrapping_add(sum)) ^ ((v1 >> 5).wrapping_add(key[1]))
        );
        v1 = v1.wrapping_add(
            ((v0 << 4).wrapping_add(key[2])) ^ (v0.wrapping_add(sum)) ^ ((v0 >> 5).wrapping_add(key[3]))
        );
    }
    v[0] = v0;
    v[1] = v1;
}

fn tea_decrypt(v: &mut [u32; 2], key: &[u32; 4]) {
    let (mut v0, mut v1) = (v[0], v[1]);
    let delta: u32 = 0x9E3779B9;
    let mut sum: u32 = delta.wrapping_mul(32);
    for _ in 0..32 {
        v1 = v1.wrapping_sub(
            ((v0 << 4).wrapping_add(key[2])) ^ (v0.wrapping_add(sum)) ^ ((v0 >> 5).wrapping_add(key[3]))
        );
        v0 = v0.wrapping_sub(
            ((v1 << 4).wrapping_add(key[0])) ^ (v1.wrapping_add(sum)) ^ ((v1 >> 5).wrapping_add(key[1]))
        );
        sum = sum.wrapping_sub(delta);
    }
    v[0] = v0;
    v[1] = v1;
}

// --- RC4 ---
struct Rc4 {
    s: [u8; 256],
    i: u8,
    j: u8,
}

impl Rc4 {
    fn new(key: &[u8]) -> Self {
        let mut s = [0u8; 256];
        for i in 0..256 {
            s[i] = i as u8;
        }
        let mut j: u8 = 0;
        for i in 0..256 {
            j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
            s.swap(i, j as usize);
        }
        Rc4 { s, i: 0, j: 0 }
    }

    fn process(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            self.i = self.i.wrapping_add(1);
            self.j = self.j.wrapping_add(self.s[self.i as usize]);
            self.s.swap(self.i as usize, self.j as usize);
            let k = self.s[(self.s[self.i as usize].wrapping_add(self.s[self.j as usize])) as usize];
            *byte ^= k;
        }
    }
}

// --- FNV-1a hash ---
fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xCBF29CE484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001B3);
    }
    hash
}

// --- Generic binary search ---
fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    let mut lo = 0usize;
    let mut hi = arr.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        match arr[mid].cmp(target) {
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Less => lo = mid + 1,
            std::cmp::Ordering::Greater => hi = mid,
        }
    }
    None
}

// --- Iterator chain test ---
fn iterator_chain_test() -> (i64, usize, i64) {
    let data: Vec<i32> = (1..=100).collect();

    let sum: i64 = data.iter()
        .filter(|&&x| x % 3 == 0 || x % 5 == 0)
        .map(|&x| x as i64 * x as i64)
        .sum();

    let count = data.iter()
        .filter(|&&x| x % 7 == 0)
        .count();

    let product: i64 = data.iter()
        .take(10)
        .fold(1i64, |acc, &x| acc * x as i64);

    (sum, count, product)
}

// --- Enum state machine ---
#[derive(Debug, PartialEq)]
enum Token {
    Number(i64),
    Plus,
    Minus,
    Star,
    End,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            '0'..='9' => {
                let mut num = 0i64;
                while let Some(&d) = chars.peek() {
                    if let Some(digit) = d.to_digit(10) {
                        num = num * 10 + digit as i64;
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num));
            }
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '-' => { tokens.push(Token::Minus); chars.next(); }
            '*' => { tokens.push(Token::Star); chars.next(); }
            ' ' => { chars.next(); }
            _ => { chars.next(); }
        }
    }
    tokens.push(Token::End);
    tokens
}

fn eval_tokens(tokens: &[Token]) -> i64 {
    let mut result = 0i64;
    let mut current_op = Token::Plus;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Number(n) => {
                match current_op {
                    Token::Plus => result += n,
                    Token::Minus => result -= n,
                    Token::Star => result *= n,
                    _ => {}
                }
            }
            Token::Plus => current_op = Token::Plus,
            Token::Minus => current_op = Token::Minus,
            Token::Star => current_op = Token::Star,
            Token::End => break,
        }
        i += 1;
    }
    result
}

fn main() -> ExitCode {
    println!("=== Rust Crypto/Iterator Tests ===");

    // TEA roundtrip
    let key = [0x01234567u32, 0x89ABCDEF, 0xFEDCBA98, 0x76543210];
    let mut data = [0xDEADBEEFu32, 0xCAFEBABE];
    let original = data;
    tea_encrypt(&mut data, &key);
    check!(data != original, "TEA encrypts");
    tea_decrypt(&mut data, &key);
    check!(data == original, "TEA roundtrip");

    // TEA: different keys produce different ciphertext
    let key2 = [0x11111111u32, 0x22222222, 0x33333333, 0x44444444];
    let mut d1 = [0xAAAAAAAAu32, 0xBBBBBBBB];
    let mut d2 = d1;
    tea_encrypt(&mut d1, &key);
    tea_encrypt(&mut d2, &key2);
    check!(d1 != d2, "TEA different keys differ");

    // RC4 roundtrip
    let rc4_key = b"secretkey";
    let plaintext = b"Hello, obfuscated world! Testing RC4 stream cipher.";
    let mut buf = plaintext.to_vec();
    Rc4::new(rc4_key).process(&mut buf);
    check!(buf != plaintext.as_slice(), "RC4 encrypts");
    Rc4::new(rc4_key).process(&mut buf);
    check!(buf == plaintext.as_slice(), "RC4 roundtrip");

    // FNV-1a
    check!(fnv1a_64(b"") == 0xCBF29CE484222325, "fnv1a empty");
    check!(fnv1a_64(b"hello") == fnv1a_64(b"hello"), "fnv1a deterministic");
    check!(fnv1a_64(b"hello") != fnv1a_64(b"world"), "fnv1a differs");

    // Binary search
    let sorted: Vec<i32> = (0..100).collect();
    check!(binary_search(&sorted, &42) == Some(42), "bsearch found");
    check!(binary_search(&sorted, &99) == Some(99), "bsearch last");
    check!(binary_search(&sorted, &100) == None, "bsearch not found");

    // String binary search
    let words = vec!["alpha", "bravo", "charlie", "delta", "echo"];
    check!(binary_search(&words, &"charlie") == Some(2), "bsearch string");
    check!(binary_search(&words, &"foxtrot") == None, "bsearch string missing");

    // Iterator chains
    let (sum, count, product) = iterator_chain_test();
    check!(sum == 200099, "iterator filter+map+sum");
    check!(count == 14, "iterator filter count div7");
    check!(product == 3628800, "iterator take(10) product = 10!");

    // Tokenizer + evaluator
    let tokens = tokenize("10 + 20 - 5");
    check!(eval_tokens(&tokens) == 25, "eval '10 + 20 - 5'");
    let tokens2 = tokenize("3 * 4 + 2");
    check!(eval_tokens(&tokens2) == 14, "eval '3 * 4 + 2'");
    let tokens3 = tokenize("100");
    check!(eval_tokens(&tokens3) == 100, "eval '100'");

    // Vec operations
    let mut v: Vec<i32> = (1..=20).collect();
    v.retain(|x| x % 2 == 0);
    let even_sum: i32 = v.iter().sum();
    check!(even_sum == 110, "vec retain+sum evens 1..20");

    // String manipulation
    let s = "Hello World Rust Test";
    let upper_count = s.chars().filter(|c| c.is_uppercase()).count();
    check!(upper_count == 4, "uppercase char count");

    let reversed: String = s.chars().rev().collect();
    let double_rev: String = reversed.chars().rev().collect();
    check!(double_rev == s, "double reverse identity");

    unsafe {
        println!("\n{} passed, {} failed", PASS, FAIL);
        if FAIL > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
    }
}
