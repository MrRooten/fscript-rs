
fn test() {
    a = "abcd efgh ijkl mnop"

    for i in a.split(" ").enumerate() {
        println("new: ", i)
    }
}

@static
fn test_jit() {
    a = "abcd efgh ijkl mnop"

    for i in a.split(" ").enumerate() {
        println("new: ", i)
    }
}

println("---------- test ----------")
test()
println("---------- test_jit ----------")
test_jit()