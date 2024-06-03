i = 0
b = 1
c = 10000000
sum = 0
while i < c {
    sum = i + sum
    i = i + b
}

println(sum)