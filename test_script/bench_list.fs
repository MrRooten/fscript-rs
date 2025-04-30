a = []

for i in 0..3000000 {
    a.push(i)
}

a.sort()

for i in 0..3000000 {
    if a[i] != i {
        assert(false, "Error at index")
    }
}

gc_info()