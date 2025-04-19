a = HashMap::new()

for i in 0..1000000 {
    a.insert(i, i)
}


for i in 0..1000000 {
    b = a[i]
    if b != i {
        assert(false)
    }
}

gc_info()