a = []

for i in 0..3000000 {
    a.push(3000000 - i)
}

a.sort_by(|x, y| {
    return x > y
})
println(a[299999])
gc_info()
