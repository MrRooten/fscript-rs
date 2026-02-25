# fn abc() {

# }

# for i in 0..18000000 {
#     abc()
# }
from itertools import repeat
def abc():
    pass

for i in repeat(True, 3000000):
    abc()