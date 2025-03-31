class Test {
    fn abc(self) {
        a = 1
	    return a
    }
}

t = Test()
t.abc()
t.index = 1
b = 1
while t.index < 3 {
    t.index = t.index + t.abc()
    t.ddc = 123 + 1
}

println(t.index)
