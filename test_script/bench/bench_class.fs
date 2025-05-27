class Test {
    fn abc(self) -> Integer {
        a = 1
	    return a
    }

    fn __add__(self, other: Test) -> Integer {
        return 1
    }
}

t: Teset = Test()
t.abc()
t.index = 1
b = 1
while t.index < 3000000 {
    t.index = t.index + t.abc()
    t.ddc = 123 + 1
    c = t + 1
}

println(t.index)

gc_info()