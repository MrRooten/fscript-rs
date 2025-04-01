fn abc3() {
    a = 1
    fn ddc() {
        a = a + 1
        println(a)
        return a
    }
    a = 3
    fn abcd() {
        return ddc
    }

    return abcd()
}

a = abc3()
println(a())