a = HashMap::new()

for i in 0..3 {
    a.insert(i, i)
}

gc_info()
a = none
gc_collect()
