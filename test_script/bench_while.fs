i = 3
while i < 3000000 { # while test
    i = i + 1
} #end of test

println(i)

dump(i)
gc_info()
i = 0
