a = [3,4,1,2,5]
println(a)

a.sort()
println(a)

a.sort_by(|x, y| {
    return true
})

str_list = ["efg", "hijk","a", "cd"]
str_list.sort_by(|x, y| {
    return x.len() > y.len()
})

println(str_list)