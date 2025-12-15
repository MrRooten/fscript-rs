
for i in (0..30).enumerate() {
    println("new: ",i)
    assert(i[0] == i[1])
}