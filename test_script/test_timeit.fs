fn abcd() {
    a = ((1 + 3 + 1 ) * 3 + 4 + 2) * 4
    return a
}

fn abc() {
    a = ((1 + 3 + 1 ) + 3 + 4 + 2) + 4
}


timeit(abc, 300000)