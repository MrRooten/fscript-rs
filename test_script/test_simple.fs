a = HashMap::new()

for i in 0..1000 {
    a.insert(i, i)
}


assert(a.contains(0))

a.remove(0)

v = a.contains(0)
assert(not v)