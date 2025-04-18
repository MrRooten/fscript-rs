a = []

for i in 0..3000000 {
    a.push(i)
}

a.sort_key(|x| {
    return x
})

a = none
gc_collect()
gc_info()
