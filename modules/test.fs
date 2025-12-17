fn test() {
    println('this is test')
}

export('test', test)

class Abc {
    fn __new__(self) {
        self.abc = 123
    }

    fn test(self) {

    }
}

export('Abc', Abc)