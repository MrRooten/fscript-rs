import iterator
import time

start = time::timestamp_ms()
c = iterator::Chain([0..1000000, 1000000..2000000])

for i in 0..100000 {
    println(i)
}

end = time::timestamp_ms()

println(f"{(end - start) / 1000}ms")