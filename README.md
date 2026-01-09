

# FScript-RS

A toy scripting language that runs on bytecode. It’s currently minimalistic and experimental, but aims to evolve into a **zero-dependency**, **embeddable**, and **Turing-complete** scripting language.
(In quick development, not going to update version number)
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

## Next main proposal(The code below is ok to run)
Support embed static type jit like subset of dynamic language.
```rust
@static
fn simple(n: u64) -> u64 {
    a: u64 = 1
    return n
}

@entry
fn test() -> u64 {
    a: u64 = 2
    b: u64 = simple(3)
    
    return b
}


a = test()
println(a)
```

Struct support
```rust
struct Test {
    field: u64
}

@entry
fn test() -> u64 {
    a: Ptr[Test] = Test.alloc
    a.field = 42
    return a.field
}

a = test()
println(a)
```
The backend uses Cranelift to compile the static typed functions to native code. The performance is at same level compared with C in some scenarios. But the implementation is still in progress.

A lot of develop designs are not determined yet.
### Memory management
The current memory management is based on threaded scope garbage collection. The static typed jit functions will use manual memory management to avoid the overhead of reference counting.


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
@static
fn simple(n: u64) -> u64 {
    a: u64 = 1
    return n
}

@entry
fn test() -> u64 {
    a: u64 = 2
    b: u64 = simple(3)
    
    return b
}


a = test()
println(a)
```

performance is 1.5-2.0x faster than origin


### Yield Support

```rust
@async
fn yield_test() {
    for i in 0..30000 {
        i.yield
    }
}

for i in yield_test() {
    println(i)
}
```

### More Examples
more examples can be found in the `test_script/` directory.


## Performance
it is still a little slower compared with Python/ruby in most scenarios. The performance will be improved in the future.
