fn a1() {
    panic("error happened")
}

fn a2() {
    a1()
}

fn a3() {
    a2()
}

fn a4() {
    a3()
}

a4()