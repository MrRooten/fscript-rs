fn abc() {
    fn fib(n) {
        if n == 1 or n == 2 {
            return 1
        } else {
            return fib(n - 1) + fib(n - 2)
        }
    }
    result = fib(30)
    println(result)
    assert(result == 832040)
    gc_info()
}

abc()