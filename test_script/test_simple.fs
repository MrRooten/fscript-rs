fn test() {
    println('abc')
    dump(println)
}

i = 0
while i < 10 {
    test()
    i = i + 1
}