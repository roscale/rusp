(fn main () {
    (+ (add (increment 1) (increment 2)) " is the result")
})

(fn increment (x) (+ x 1))
(fn add (first second) (** first second))
