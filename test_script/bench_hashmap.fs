a = HashMap::new()

for i in 0..1000000 {
    a.insert(i, i)
}

gc_collect()

gc_info()