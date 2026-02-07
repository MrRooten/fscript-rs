class Chain {
    fn __new__(self, chain_iter) {
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
                self.index = self.index + 1
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

fn test_chain() {
    c = iterator::Chain([[1,2,3], [4,5,6]])

    for i in c {
        println(i)
    }
}