fn index() {
    return 1
}

a = [1, 2, 3, 4, 5]
println(a[0 + 2])
assert(a[1] == 2, "list_getter: testcase1: a[1] should be 2")

println(a[index()])

a[1] = 100 + 2
assert(a[1] == 102, "list_getter: testcase2: a[1] should be 102")

b = [[1,2,3], [4,5,6]]
c = b[1][2]
assert(c == 6, "list_getter: testcase3: c should be 6")