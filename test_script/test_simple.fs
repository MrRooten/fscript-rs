a = 2
a = a.then(|x| {
    println("Hello World")
    return 2
}).map_err(|x| {
    println("Error happened")
    return 1
})

println(a)
