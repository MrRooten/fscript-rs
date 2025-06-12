

# FScript-RS

A toy scripting language that runs on bytecode. It’s currently minimalistic and experimental, but aims to evolve into a **zero-dependency**, **embeddable**, and **Turing-complete** scripting language.

---

## 🚀 Features & Roadmap

| Feature                   | Status |
| ------------------------- | ------ |
| Float type                | ✅      |
| Async support             | ❌      |
| Module import             | ✅      |
| Type hint system          | ❌      |
| Better AST error messages | ❌      |
| JIT compiler              |[partial]|
| Closure support           | ✅      |
| Anonymous functions       | ✅      |
| Class support             | ✅      |
| Static typed jit support  | ❌      |
| Coroutines support        | ❌      |

---

## 🔧 Build Instructions

```bash
cargo build --release
```

---

## Test
Run the test script to see the language in action:

thread is 1 to avoid some thread issues.(fix later)
```bash
cargo test --release -- --test-threads=1
```

## 🧪 Examples

### 🔁 While Loop

```rust
i = 3
while i < 3000000 {
    i = i + 1
}

println(i)
```

Run:

```bash
target/release/fscript-rs ./test_script/test_while.fs
```

Output:

```
3000000
count: 26999986
obj count: 4
184.098625ms
```

---

### 🧱 Class Usage

```python
class Abc {
    fn __new__(self) {
        self.abc = 123
        return self
    }

    fn __str__(self) {
        return 'return string'
    }
}

a = Abc()
println(a)

println("Get a.abc")
println(a.abc)

if a.abc > 3 {
    println('a.abc > 3')
}
```

Output:

```
return string
Get a.abc
123
a.abc > 3
count: 44
675.794µs
```

---

### 🔃 For Loop

```rust
a = [1, 2, 3, 4, 5]

for i in a {
    println(i)
}
```

Output:

```
1
2
3
4
5
count: 79
41.709µs
```

---

### Chain iterator

```rust
for i in (0..30).map(|x| {
    return x * 2
}).filter(|x| {
    return x % 3 == 0
}) {
    println("new:",i)
}
```

### 📦 Module Import

> Modules are located in the `modules/` directory.

```python
import test

test.test()

abc = test.Abc()

dump(test.Abc)
```


### JIT Support
```rust
class Abc {
    @jit
    fn __add__(self, other: Abc) -> Integer {
        #println("add")
        return 1
    }
}

@jit
fn jit_test(n) {
    for i in 0..30000000 {
        n + n
    }
}

a = Abc()
jit_test(a)
```

performance is 1.5-2.0x faster than origin

### More Examples
more examples can be found in the `test_script/` directory.


## Performance
it is still a little slower than most scenarios compared with Python/ruby. The performance will be improved in the future.
