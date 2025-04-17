a = []

for i in 0..300000 {
    a.push(300000 - i)
}

a.sort_by(|x, y| {
    return x > y
})
println(a[299999])
gc_info()
