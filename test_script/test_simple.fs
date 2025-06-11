import fs

file = fs::File.open(".gitignore")

line_count = file.lines().count()

println(line_count)