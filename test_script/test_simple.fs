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
for i in 0..3000000 {
    t.abc()
}