fn test(i) {
    return i % 2
}

for i in 0..3000000 {
    if test(i) {
        continue
    }
}