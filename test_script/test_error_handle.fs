fn abc() {
    try {
        throw_error(1)
    } catch {
        println('in fn abc catch')
    }
    
    println('in fn abc ,this is normal')
}
try {
    a = 1 == 1
    abc()
    println('if not error will print')
    assert(true)
} catch {
    e = get_error()
    println(e)
    println("catch")
    assert(false)
}

println('ok')