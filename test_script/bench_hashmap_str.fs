t = HashMap::new()
for i in 0..3000000 {
    t.insert(i, i)
}

gc_info()