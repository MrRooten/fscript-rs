fn abcd() {
    a = ((1 + 3 + 1 ) * 3 + 4 + 2) * 4
    return a
}

fn abc() {
    c = abcd()
    a = ((1 + 3 + 1 ) * 3 + 4 + 2) * 4 + c
    return a
}

a = abc()

i = 0
while i < 3 {
    i = i + 1
    abc()
}