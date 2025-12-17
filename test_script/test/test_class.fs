class Ddc {
    fn __new__(self) {
        self.ddc = 123 + 1
    }
}

class Abc {
    fn __new__(self) -> Abc {
        self.abc = Ddc()
    }

    fn __str__(self) -> String {
        return 'return string'
    }

    fn test(self) -> Integer {
        return 323
    }

    fn not_self(abc) {
        println("not self: ", abc)
        return 1
    }

    fn __add__(self, other: Abc) -> Integer {
        println("add")
        return 1
    }

    fn __gt__(self, other) {
        println("gt")
        return true
    }

    fn more_args(self, a, b, c) {
        assert(a == 1)
        assert(b == 2)
        assert(c == 3)
        println("more args", a, b, c)
        return 1
    }
}



a = Abc()
println(a.__str__()) # will prin 'return string'


if a.abc.ddc < 323 {
    a.abc.ddc = a.test() + a.abc.ddc
}

assert(a.abc.ddc == 447)

println(a.abc.ddc)

Abc::not_self(1)

println(a)

println(a.more_args(1, 2, 3))

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
assert(a1[0].value == 1)
assert(a1[4].value == 5)

class Test {
    fn __new__(self, a, b, c) {
        println(f"a: {a}, b: {b}, c: {c}")
        assert(a == 1)
        assert(b == 2)
        assert(c == 3)
        return self
    }
}

a = Test(1, 2, 3)