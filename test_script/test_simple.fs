fn abc() {
    "abc".raise
}

try {
    abc()
} catch {
    err = take_error()
    println(err)
}
