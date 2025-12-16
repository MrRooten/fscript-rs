class Test {
    fn abc(self) -> Integer {
	    
    }

    fn __add__(self, other: Test) -> Integer {
        return 1
    }
}

t: Teset = Test()
t.abc()
t.index = 1
b = 1
f = t.abc
for i in 0..3000000 {
    t.abc()
}

println(t.index)

gc_info()