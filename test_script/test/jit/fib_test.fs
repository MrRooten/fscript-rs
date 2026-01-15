@static
fn fib(n: u64) -> u64 {
    if n <= 1 {
        return n
    }

    return fib(n - 1) + fib(n - 2)
}

@entry
fn test() -> u64 {
    b: u64 = fib(35)
    return b
}

a = test()
println(f"a: {a}")
assert(a == 9227465, "simple: testcase1: a should be 9227465")