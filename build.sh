RUSTFLAGS="
  -C target-cpu=native
  -C codegen-units=1
  -C force-frame-pointers=yes
" cargo build --release