a = (0..10).__iter__().map(|x| {
    println("ori: ", x)
    return x + 1
})

for i in a {
    println("new:",i)
}