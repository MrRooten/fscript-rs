# RUSTFLAGS="
#   -C target-cpu=native
#   -C force-frame-pointers=yes
# " /usr/bin/time -v cargo +nightly build --release 


RUSTFLAGS="
  -C target-cpu=native
  -C force-frame-pointers=yes
" /usr/bin/time -v cargo +nightly build --release 