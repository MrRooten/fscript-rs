RUSTFLAGS="
  -C target-cpu=native
  -C force-frame-pointers=yes
" /usr/bin/time -v cargo build --release 