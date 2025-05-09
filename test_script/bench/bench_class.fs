class Test {
    fn abc(self) {
        a = 1
	    return a
    }

    fn __add__(self, other) {
        return 1
    }
}

t = Test()
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