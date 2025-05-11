
b = 0

for i in (0..3000000).filter(|x| { return x % 2 == 0 }) {
    b = b + 1
}

println("count filter:",b)