fn abc() {
    a = 1
    b = 1
    ddc = || {
        return a + b
    }

    return ddc
}



fn main() {
    a = abc()

    c = a()
    println(c)
}

main()