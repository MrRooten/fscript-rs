fn test() {
    return 'test fn'
}

a = [1, 2, 'abc', 4, 5, test(), 'abc'.len()]

for i in [1, 2, 3, 4, 6] {

    c = 0
    while c < 10 {
        c = c + 1
        if c > 4 {
            break
        }
        print("this is inner loop: ")
        println(c)
    }

    if i == 3 {
        println("no 3")
        continue
    }

    println(i)

    if i > 4 {
        break
    }
    
}
