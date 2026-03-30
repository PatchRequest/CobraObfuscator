fn fib(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

fn classify(x: i32) -> &'static str {
    match x {
        i32::MIN..=-1 => "negative",
        0 => "zero",
        1..=i32::MAX => "positive",
    }
}

fn sum_iter(n: u32) -> u32 {
    (1..=n).sum()
}

fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

fn count_vowels(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

fn main() {
    println!("fib(10)={}", fib(10));
    println!("classify: {}={} {}={} {}={}",
             -5, classify(-5), 0, classify(0), 42, classify(42));
    println!("sum(100)={}", sum_iter(100));
    println!("rev={}", reverse_string("Hello"));
    println!("vowels={}", count_vowels("CobraObfuscator"));
}
