// Rust test: trait objects, enums with data, pattern matching, Box/Vec,
// error handling, string formatting — exercises Rust-specific codegen patterns.

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

// --- Trait objects (dynamic dispatch) ---
trait Animal {
    fn name(&self) -> &str;
    fn legs(&self) -> u32;
    fn sound(&self) -> &str;
}

struct Dog { breed: String }
struct Cat;
struct Spider;

impl Animal for Dog {
    fn name(&self) -> &str { &self.breed }
    fn legs(&self) -> u32 { 4 }
    fn sound(&self) -> &str { "woof" }
}

impl Animal for Cat {
    fn name(&self) -> &str { "cat" }
    fn legs(&self) -> u32 { 4 }
    fn sound(&self) -> &str { "meow" }
}

impl Animal for Spider {
    fn name(&self) -> &str { "spider" }
    fn legs(&self) -> u32 { 8 }
    fn sound(&self) -> &str { "..." }
}

fn total_legs(animals: &[Box<dyn Animal>]) -> u32 {
    animals.iter().map(|a| a.legs()).sum()
}

// --- Enum with data (tagged union) ---
#[derive(Debug, PartialEq)]
enum Expr {
    Num(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Num(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Mul(a, b) => eval(a) * eval(b),
        Expr::Neg(e) => -eval(e),
    }
}

fn expr_depth(expr: &Expr) -> usize {
    match expr {
        Expr::Num(_) => 1,
        Expr::Add(a, b) | Expr::Mul(a, b) => 1 + expr_depth(a).max(expr_depth(b)),
        Expr::Neg(e) => 1 + expr_depth(e),
    }
}

// --- Result/Option chains ---
fn parse_and_sum(inputs: &[&str]) -> Result<i64, String> {
    let mut total = 0i64;
    for &s in inputs {
        let n: i64 = s.parse().map_err(|e| format!("parse error: {}", e))?;
        total += n;
    }
    Ok(total)
}

fn safe_divide(a: i64, b: i64) -> Option<i64> {
    if b == 0 { None } else { Some(a / b) }
}

// --- Linked list via enum ---
enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),
}

impl<T: Copy + std::ops::Add<Output = T> + Default> List<T> {
    fn push(self, val: T) -> Self {
        List::Cons(val, Box::new(self))
    }

    fn sum(&self) -> T {
        match self {
            List::Nil => T::default(),
            List::Cons(val, next) => *val + next.sum(),
        }
    }

    fn len(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(_, next) => 1 + next.len(),
        }
    }

    fn to_vec(&self) -> Vec<T> {
        let mut v = Vec::new();
        let mut current = self;
        loop {
            match current {
                List::Nil => break,
                List::Cons(val, next) => {
                    v.push(*val);
                    current = next;
                }
            }
        }
        v
    }
}

// --- Generic stack ---
struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self { Stack { data: Vec::new() } }
    fn push(&mut self, val: T) { self.data.push(val); }
    fn pop(&mut self) -> Option<T> { self.data.pop() }
    fn len(&self) -> usize { self.data.len() }
    fn is_empty(&self) -> bool { self.data.is_empty() }
}

// --- String builder pattern ---
struct Builder {
    parts: Vec<String>,
}

impl Builder {
    fn new() -> Self { Builder { parts: Vec::new() } }
    fn add(mut self, s: &str) -> Self { self.parts.push(s.to_string()); self }
    fn build(self) -> String { self.parts.join(" ") }
}

fn main() -> ExitCode {
    println!("=== Rust Struct/Trait/Enum Tests ===");

    // Trait objects
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog { breed: "labrador".to_string() }),
        Box::new(Cat),
        Box::new(Spider),
    ];
    check!(animals[0].name() == "labrador", "dog name");
    check!(animals[1].sound() == "meow", "cat sound");
    check!(animals[2].legs() == 8, "spider legs");
    check!(total_legs(&animals) == 16, "total legs");

    // Expression tree
    // (3 + 4) * -(2 + 5) = 7 * -7 = -49
    let expr = Expr::Mul(
        Box::new(Expr::Add(Box::new(Expr::Num(3)), Box::new(Expr::Num(4)))),
        Box::new(Expr::Neg(Box::new(Expr::Add(
            Box::new(Expr::Num(2)),
            Box::new(Expr::Num(5)),
        )))),
    );
    check!(eval(&expr) == -49, "expr eval (3+4)*-(2+5)");
    check!(expr_depth(&expr) == 4, "expr depth");

    // Simple expressions
    check!(eval(&Expr::Num(42)) == 42, "expr literal");
    check!(eval(&Expr::Neg(Box::new(Expr::Num(10)))) == -10, "expr neg");

    // Result chains
    check!(parse_and_sum(&["1", "2", "3"]) == Ok(6), "parse_and_sum ok");
    check!(parse_and_sum(&["1", "abc", "3"]).is_err(), "parse_and_sum err");

    // Option chains
    check!(safe_divide(10, 3) == Some(3), "safe_divide ok");
    check!(safe_divide(10, 0) == None, "safe_divide by zero");
    let chained = safe_divide(100, 5)
        .and_then(|x| safe_divide(x, 4))
        .unwrap_or(-1);
    check!(chained == 5, "option chain divide");

    // Functional linked list
    let list = List::Nil.push(1).push(2).push(3).push(4).push(5);
    check!(list.len() == 5, "list len");
    check!(list.sum() == 15, "list sum");
    let v = list.to_vec();
    check!(v == vec![5, 4, 3, 2, 1], "list to_vec (reversed)");

    // Generic stack
    let mut stack: Stack<i32> = Stack::new();
    for i in 0..10 {
        stack.push(i * i);
    }
    check!(stack.len() == 10, "stack len");
    check!(stack.pop() == Some(81), "stack pop top");
    check!(stack.pop() == Some(64), "stack pop next");
    check!(stack.len() == 8, "stack len after pops");

    // String stack
    let mut sstack: Stack<String> = Stack::new();
    sstack.push("hello".to_string());
    sstack.push("world".to_string());
    check!(sstack.pop().unwrap() == "world", "string stack pop");

    // Builder pattern
    let sentence = Builder::new()
        .add("the")
        .add("quick")
        .add("brown")
        .add("fox")
        .build();
    check!(sentence == "the quick brown fox", "builder pattern");

    // Vec operations
    let mut nums: Vec<i32> = (1..=20).collect();
    nums.sort_by(|a, b| b.cmp(a)); // reverse sort
    check!(nums[0] == 20, "reverse sort first");
    check!(nums[19] == 1, "reverse sort last");

    let evens: Vec<i32> = nums.iter().filter(|&&x| x % 2 == 0).cloned().collect();
    check!(evens.len() == 10, "filter evens count");
    let even_sum: i32 = evens.iter().sum();
    check!(even_sum == 110, "even sum");

    // String formatting
    let formatted = format!("{:08X}", 0xDEADBEEFu32);
    check!(formatted == "DEADBEEF", "hex format");

    let padded = format!("{:>10}", "hello");
    check!(padded == "     hello", "right-pad format");

    unsafe {
        println!("\n{} passed, {} failed", PASS, FAIL);
        if FAIL > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
    }
}
