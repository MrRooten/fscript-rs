i = 3
while i < 3000000 { # while test
    i = i + 1
} #end of test

println(i)

dump(i)

i = 0
gc_collect()
gc_info()