a = [1, 2, 3, 4, 5]
for i in 0..3000000 {
    a[0] = 101
}

println(a)