@async
fn abc() {
    for i in 0..300 {
        i.yield
    }
}

v = 0
for i in abc() {
    assert(i == v)
    v = v + 1
}