fn fn_cost() {

}

@jit
fn fib(n) {
    for i in 0..30000000 {
        fn_cost()
    }
}


fib(1)