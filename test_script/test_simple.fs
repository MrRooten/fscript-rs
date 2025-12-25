fn abc() {
    a = 1
    b = 1
    ddc = || {
        a += 1
        return a
    }

    return ddc
}

a = abc()
println(a())