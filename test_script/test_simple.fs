(0..3000000).__iter__().map(|x| {
    println("a.map: ", x)
    return x + 1
})