fn abc() {
    throw_error(1)
    
    println('in fn abc ,this is normal')
}

i = 0
while i < 3 {
    try {
        abc()
        println('should not print')
        assert(false)
    } catch {
        e = take_error()
        println(e)
        println("catch")
    }

    i = i + 1
}


println('ok')