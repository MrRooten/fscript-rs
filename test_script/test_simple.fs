@static
fn test(n: i64) {
    i: u64 = 0
    while i < n {
        i = i + 1
    }
}

test(10000000)