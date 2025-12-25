fn abc() {
    a = 1
    ddc = || {
        a = a + 1
        println(a)
    }

    return ddc
}

a = abc()
a()