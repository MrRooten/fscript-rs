@jit
fn abc() {
    i = 1 + 1
}

for i in 0..30000000 {
    abc()
}
gc_info()