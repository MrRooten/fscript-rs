
fn test() {
    println("test")
    return true
}

@jit
fn abc(n) {
    if n == 0 or n == 1 {
        return n
    }

    return abc(n - 1) + abc(n - 2)
}


println(abc(17))
gc_info()