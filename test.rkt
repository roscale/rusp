fn demo (a) {
    fn increment (x) (+ x a)
    (+ (add (increment 1) (increment 2)) " is the result")
}

fn add (first second) (** first second)

let arg = 2
(demo arg)