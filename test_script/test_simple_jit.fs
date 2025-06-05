
fn test() {
    println("test")
    return true
}


fn abc(n) {
    if n == 0 or n == 1 {
        return 1
    }

    return 3
}

for i in 0..3000000 {
    abc(0)
}