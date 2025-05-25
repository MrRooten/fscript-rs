
a = [0, 1, 2, 3].__iter__().any(|x| { return x == 2 })
assert(a == true)