a = (0..10).__iter__()

new_a = a.map(|x| {
    println("a.map: ", x)
    return x + 1
})

for i in new_a {
    println(i)
}