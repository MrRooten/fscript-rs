import collection

c = collection::Counter([1, 2, 3, 2, 3, 3, 5,1,1,1,1])
v = c.most_common(2)

println(v)