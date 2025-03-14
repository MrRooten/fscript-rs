a = 1
println(type(a))

class Abc {
    fn __new__(self) {
        return self
    }
}

a = Abc()
println(type(a))

print("ref_count:")
println(ref_count(a))

println(println)