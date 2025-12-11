class Test {
    fn abc(self) -> Integer {
        println("----------------------------")
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

fn abc() {
    a = 1
    return a
}

c = Test::abc

while t.index < 300000 {
    println(t.abc())
    t.index = t.index + 1
    println("+++++++++++++++++++++++++++++")
}

println(t.index)

gc_info()