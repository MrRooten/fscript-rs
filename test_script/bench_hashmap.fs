a = HashMap::new()

for i in 0..1000000 {
    a.insert(i, "abc")
}


for i in 0..1000000 {
    b = a[i]
    if b != "abc" {
        assert(false)
    }
}

gc_info()