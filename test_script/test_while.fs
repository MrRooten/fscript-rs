i = 0
while i < 3000000 {
    i = 1 + i

    if i == 50000 {
        println('i == 500000 continue')
        i = i + 1
        continue
    }

    if i == 392000 {
        println('i == 392000')
        break
    }

    
}

println(i)