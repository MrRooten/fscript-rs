@jit
fn abc() {
    i = 1 + 1
}

for i in 0..3000000 {
    abc()
}
gc_info()