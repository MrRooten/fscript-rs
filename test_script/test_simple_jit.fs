
fn test() {
    println("test")
    return true
}

@jit
fn abc(n) {
    i = 0
    while i < 3000000 {
        i = i + 1
    }
}


println(abc(16))
gc_info()