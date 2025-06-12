
c = (0..30).filter(|x| {
    return x % 2 == 0
}).count()

println(c)

assert(c == 15)

