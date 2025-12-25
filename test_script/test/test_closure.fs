fn abc() {
    a = 1
    b = 1
    ddc = || {
        return a + b
    }

    return ddc
}

fn abc2() {
    fn ddc() {
        return 1
    }

    fn abcd() {
        return ddc
    }

    return abcd()
}

a = abc2()
println(a)

assert(a() == 1)


a = abc()

assert(a() == 2)

fn abc3() {
    a = 1
    fn ddc() {
        a = a + 1
        println(a)
        return a
    }

    fn abcd() {
        return ddc
    }

    return abcd()
}

a = abc3()
assert(a() == 2, "a() == 2 error")

fn abc4() {
    a = 1
    fn ddc() {
        a = a + 1
        println(a)
        return a
    }

    fn abcd() {
        return ddc
    }

    return abcd()
}

dd = abc4()
assert(dd() == 2, "a() == 2 error")