for i in (0..30).filter(|x| {
    return x % 2 == 0
}) {
    println("new:",i)
}