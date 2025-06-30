def abc():
	i = 0
	for i in range(3000000):
		yield i

for i in abc():
	pass
