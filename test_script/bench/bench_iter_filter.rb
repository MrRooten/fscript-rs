
# for i in (0..3000000).filter(|x| {
#     if x % 2 == 0 {
#         return true
#     }

#     return false
# }) {
    
# }

def process_even_number(x)
    # Perform your logic here for even numbers
end

(0..3000000).each do |x|
    process_even_number(x) if x % 2 == 0
end

