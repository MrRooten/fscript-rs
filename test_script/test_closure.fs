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