# n = 2
# for i in range(3000000):
#     if i % 2 == 0:
#         pass

n = 2
filtered = filter(lambda i: i % n == 0, range(3000000))
for _ in filtered:
    pass