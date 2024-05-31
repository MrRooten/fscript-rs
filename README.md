## Introduce
Running on bytecode, simple and useless program language, zero dependence script language.
## Usage

### Compile

```bash
cargo build --release
```

### Test
```bash
target/release/fscript-rs ./test_script/test_while.fs
```

```
test and benchmark self add 3000000
3000000
count: 27000012
522.127195ms
```