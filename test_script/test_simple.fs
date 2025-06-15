a = HashSet::new()

a.insert(1)
a.insert(2)
a.insert(1)
println(a)

println("----- remove item test -----")
a.remove(1)
for i in a {
    println(i)
}