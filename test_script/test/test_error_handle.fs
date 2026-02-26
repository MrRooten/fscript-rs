fn abc() {
    try {
        Exception().raise
        assert(false, 'should not reach here')
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
    e = take_error()
    println(e)
    println("catch")
    assert(false)
}

println('ok')