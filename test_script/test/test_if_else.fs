abc = -2

while abc < 3 {
    if abc > 1 {
        println(' > 1: ', abc)
    } else if abc < -1 {
        println(' < -1: ', abc)
    } else {
        println('else:', abc)
    }
    abc = abc + 1
}
