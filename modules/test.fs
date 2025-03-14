fn test() {
    println('this is test')
}

export('test', test)

class Abc {
    fn __new__(self) {
        self.abc = 123
        return self
    }

    fn test(self) {

    }
}

export('Abc', Abc)