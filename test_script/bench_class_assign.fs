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
while t.index < 3000000 {
    t.index = t.index + 1
}

println(t.index)
gc_info()