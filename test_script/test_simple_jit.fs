fn fn_cost() {

}

@jit
fn fib(n) {
    a = fn_cost
    for i in 0..30000000 {
        a()
    }
}


fib(1)