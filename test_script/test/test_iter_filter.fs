a = (0..10).__iter__().filter(|x| {
    if x % 2 == 0 {
        return true
    }

    return false
})

for i in a {
    println("new:",i)
}