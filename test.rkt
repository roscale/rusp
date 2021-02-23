fn sum (from to) {
    let sum = 0
    let i = from
    while (<= i to) {
        sum = (+ sum i)
        i = (+ i 1)
    }
    sum
}

fn max (x y) if (> x y) x else y
let min = fn (x y) if (< x y) x else y

let is_greater_than = fn (x) fn (y) (> y x)
let is_greater_than_4 = (is_greater_than 4)

((is_greater_than 10) 11)
(is_greater_than_4 3)

; (= (sum 1 4) 10)
; (max 3 2)