RUSTFLAGS="
  -C target-cpu=native
  -C codegen-units=1
  -Cforce-frame-pointers=yes
" cargo build --release