class Chain {
    fn __new__(self, chain_iter: List[Iterator]) {
        self.chains = chain_iter.map(|x| {
            return x.__iter__()
        })
        self.index = 0
    }

    fn __next__(self) {
        len = self.chains.len()

        if self.index < len {
            iter = self.chains[self.index]
            v = iter.__next__()
            while v == none and self.index < len {
                self.index += 1
                if self.index == len {
                    return none
                }
                iter = self.chains[self.index]
                v = iter.__next__()
            }

            return v
        }

        return none
    }
}

class Zip {
    fn __new__(self, list_iterator) {
        self.list_iterator = list_iterator.map(|x| {
            return x.__iter__()
        })
    }

    fn __next__(self) {
        ret = []
        any_value = false

        for item in self.list_iterator {
            value = item.__next__()
            ret.push(value)
            # push_fn(ret, value)
            if value != none {
                any_value = true
            }
        }

        if !any_value {
            return none
        }

        return ret
    }

}

class Skip {
    fn __new__(self, iter, n) {
        self.iter = iter.__iter__()
        self.n = n
    }

    fn __next__(self) {
        for i in 0..self.n {
            v = self.iter.__next__()
            if v == none {
                return none
            }
        }

        return self.iter.__next__()
    }
}

fn test_chain() {
    c = iterator::Chain([[1,2,3], [4,5,6]])

    for i in c {
        println(i)
    }
}

fn test_zip() {
    a = [[1,2,3], [4,5,6]]
    z = iterator::Zip(a)

    for i in z {
        println(i)
    }
}

fn test_skip() {
    a = [1,2,3,4,5]
    s = iterator::Skip(a, 2)

    for i in s {
        println(i)
    }
}