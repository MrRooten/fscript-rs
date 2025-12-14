@async
fn abc() {
    for i in 0..3000000 {
        i.yield
    }
}

v = 0
for i in abc() {
    #assert(i == v)
    #v = v + 1
}