for i in (0..30).map(|x| {
    return x * 2
}).filter(|x| {
    return x % 3 == 0
}).map(|x| {
    return x + 1
}) {
    println("new:",i)
}