a = HashMap::new()

for i in 0..100 {
    a.insert(i, i)
}

for i in 0..100 {
    b = a[i]
}

println(a)