i = 0
while i < 300000 {
    i = 1 + i

    if i == 50000 {
        println('i == 50 continue')
        i = i + 1
        continue
    }

    if i == 39200 {
        println('i == 392')
        break
    }

    
}

println(i)