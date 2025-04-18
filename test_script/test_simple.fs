a = HashMap::new()

for i in 0..1000000 {
    a.insert(i, i)
}

dump(a)

for i in 0..1000000 {
    #println(i)
    b = a.get(i)
    
    if b != i {
        assert(false)
    }
}

gc_info()