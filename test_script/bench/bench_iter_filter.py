
# for i in (0..3000000).filter(|x| {
#     if x % 2 == 0 {
#         return true
#     }

#     return false
# }) {
    
# }

for i in filter(lambda x: x % 2 == 0, range(3000000)):
    pass