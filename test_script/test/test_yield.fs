@async
fn abc() {
    for i in 0..3 {
        res = i.yield
        println(f"send value: {res}")
    }
}

v = 0
for i in abc() {
    assert(i == v)
    v = v + 1
}

new_res = abc()
new_res.send(1)
for i in new_res {
    println(i)
}