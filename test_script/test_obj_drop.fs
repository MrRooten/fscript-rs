fn abcd() {
    a = ((1 + 3 + 1 ) * 3 + 4 + 2) * 4
    return a
}

fn abc() {
    c = abcd()
    a = ((1 + 3 + 1 ) * 3 + 4 + 2) * 4 + c
}

i = 1
while i < 30000 {
    abc()
    i = i + 1
}