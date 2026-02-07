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
println(f"a() = {a()}")
assert(a() == 2, "abc() error")

fn abc3() {
    a = 1
    ddc = || {
        a = a + 1
        assert(a == 2)
        println(f"a in ddc from abc3: {a}")
        return a
    }

    fn abcd() {
        return ddc
    }

    return abcd()
}

a = abc3()
assert(a() == 2, "a = a + 1: a() == 2 error")

fn abc4() {
    a = 1
    fn ddc() {
        a += 1
        println(f"a in ddc from abc4: {a}")
        assert(a == 2)
        return a
    }

    fn abcd() {
        return ddc
    }

    return abcd()
}

a = abc4()
assert(a() == 2, "+= a() == 2 error")