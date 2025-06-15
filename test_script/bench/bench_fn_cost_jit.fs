@jit
fn abc() {

}

for i in 0..3000000 {
    abc()
}

gc_info()