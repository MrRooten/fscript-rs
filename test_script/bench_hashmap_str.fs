t = HashMap::new()
for i in 0..3000000 {
    a = str(i)
    t.insert(a, i)
}

gc_info()