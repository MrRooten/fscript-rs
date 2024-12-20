## Introduce
Running on bytecode, simple and useless program language, zero dependence script language.

## Wait for support
### [x] float type
### [x] async 
### [x] import module



## Usage

### Compile

```bash
cargo build --release
```

### Test
#### While Test
```rust
println('test and benchmark self add 3000000')

i = 0
while i < 3000000 {
    i = i + 1
}

println(i)
```

```bash
target/release/fscript-rs ./test_script/test_while.fs
```

```
test and benchmark self add 3000000
3000000
count: 27000012
397.871106ms
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