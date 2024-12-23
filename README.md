## Introduce
Running on bytecode, simple and useless program language, zero dependence script language.

## Wait for support
### [x] float type
### [x] async support 
### [x] import module


## Usage

### Compile

```bash
cargo build --release
```

### Test
#### While Test
```rust
i = 3
one = 1
b = 3000000
while i < b {
    i = i + one
}

println(i)
```

```bash
target/release/fscript-rs ./test_script/test_while.fs
```

```
3000000
count: 26999986
obj count: 4
294.098625ms
```

#### Class Test
```rust
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

```
return string
Get a.abc
123
a.abc > 3
count: 44
675.794µs
```

#### For Test
```rust
a = [1, 2, 3, 4, 5]

for i in a {
    println(i)
}
```

```
1
2
3
4
5
count: 79
41.709µs
```