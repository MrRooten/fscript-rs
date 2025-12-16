RUSTFLAGS="
  -C target-cpu=native
  -C codegen-units=1
" cargo build --release