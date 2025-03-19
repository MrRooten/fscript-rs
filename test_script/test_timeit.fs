fn abcd() {
    a = ((1 + 3 + 1 ) * 3 + 4 + 2) * 4
    return a
}

fn abc() {
    a = ((1 + 3 + 1 ) * 3 + 4 + 2) * 4
    return a
}


i = 0 
while i< 3000000 {
    i = i + 1
    abc()
}