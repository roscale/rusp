pub trait Boxed {
    fn boxed(self) -> Box<Self>;
}

impl<T> Boxed for T {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

pub fn skip<T>(slice: &mut &[T], n: usize) {
    *slice = &slice[n..];
}

pub fn next<T>(slice: &mut &[T]) {
    skip(slice, 1);
}
