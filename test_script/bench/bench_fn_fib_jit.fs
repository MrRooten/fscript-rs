
@jit
fn fib(n) {
    n = 2
    if n == 1 or n == 2 {
        return 1
    }
    return fib(n - 1) + fib(n - 2)
    
}

for i in 0..18000000 {
    fib(2)
}


