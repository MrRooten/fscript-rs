fn abc() {
    throw_error(1)
    
    println('in fn abc ,this is normal')
}

i = 0
while i < 3 {
    try {
        timeit(abc, 1)
        println('should not print')
        assert(false)
    } catch {
        e = get_error()
        println(e)
        println("catch")
    }

    i = i + 1
}


println('ok')