
fn fib(n) {
    if n == 1 or n == 2 {
        return 1
    } else {
        return fib(n - 1) + fib(n - 2)
    }
}

a = 1

for i in 0..18000 {
    println(i)
    fib(2)
}
