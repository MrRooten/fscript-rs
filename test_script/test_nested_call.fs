fn abc(c) {
    println("cur value")
    println(c)
    i = c
    if i > 3 {
        return i
    } else {
        return abc(c + 1)
    }
    return 1
}

println(abc(0))