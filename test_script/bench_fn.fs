fn abc() {
    a = ((1 + 3 + 1 ) + 3 + 4 + 2) + 4
}

b = 1
while b < 3000000 {
    abc()
    b = b + 1
}

gc_info()
gc_collect()
gc_info()