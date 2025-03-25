fn abc() {
    a = 1
    b = 1
    fn ddc() {
        return a + b
    }

    return ddc
}

a = abc()

println(a())