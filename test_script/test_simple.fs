import fs 

file = fs::File.open("./.gitignore", "b")

data = file.read_all()

println(data.len())