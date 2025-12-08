@async
fn abc() {
    for i in 0..300 {
        breakpoint()
        i.yield
    }
}

for i in abc() {
    println(i)
}