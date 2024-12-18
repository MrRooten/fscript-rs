fn test() {
    return 'test fn'
}

a = [1, 2, 'abc', 4, 5, test(), 'abc'.len()]

for i in a {
    println(i)
}