@jit
fn abc() {
    i = 0
    while i < 30000000 {
        i = i + 1
    }
}

println(abc())
gc_info()