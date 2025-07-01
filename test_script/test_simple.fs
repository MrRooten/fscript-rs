@async
fn abc() {
    for i in 0..30000000 {
        i.yield
    }
}

for i in abc() {
    
}