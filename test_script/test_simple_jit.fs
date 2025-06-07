
@jit
fn fib(n) {
    i = 0
    while i < 3000000 {
        i = i + 1
    }
    #println("fib:", i)
}

fib(1)