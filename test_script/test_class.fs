class Ddc {
    fn __new__(self) {
        self.ddc = 123 + 1
        return self
    }
}

class Abc {
    fn __new__(self) {
        self.abc = Ddc()
        return self
    }

    fn __str__(self) {
        return 'return string'
    }

    fn test(self) {
        return 323
    }

    fn not_self() {
        println("not self")
        return 1
    }

    fn __add__(self, other) {
        println("add")
        return 1
    }

    fn __gt__(self, other) {
        println("gt")
        return true
    }
}



a = Abc()
println(a.__str__()) # will prin 'return string'


if a.abc.ddc < 323 {
    a.abc.ddc = a.test() + a.abc.ddc
}

assert(a.abc.ddc == 447)

println(a.abc.ddc)

Abc::not_self()

println(a)

assert(is_class(a, Abc))

class SortItem {
    fn __new__(self, value) {
        self.value = value
        return self
    }

    fn __gt__(self, other) {
        return self.value > other.value
    }

    fn __str__(self) {
        return "SortItem" + "(" + str(self.value) + ")"
    }
}

a1 = [SortItem(3), SortItem(4), SortItem(1), SortItem(2), SortItem(5)]
a1.sort()
println("sort class")
println(a1)
println(a1[0].value == 1)
println(a1[4].value == 5)