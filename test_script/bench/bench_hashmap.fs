a = HashMap::new()

for i in 0..1000000 {
    a.insert(i, i)
}

println("abc")

for i in 0..1000000 {
    a[i]
}

gc_info()