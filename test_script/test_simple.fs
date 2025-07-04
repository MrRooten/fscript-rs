@async
fn yield_test() {
    for i in 0..30000 {
        i.yield
    }
}

for i in yield_test() {
    println(i)
}