@entry
fn test() -> u64 {
    t: [u64, 4] = uninit
    t[0] = 1
    b: u64 = 1
    return b
}

test()