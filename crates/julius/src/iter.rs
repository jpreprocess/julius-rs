pub struct BindIterator<T, W> {
    current: *mut T,
    get_next: Box<dyn Fn(*mut T) -> *mut T>,
    ctor: Box<dyn Fn(*mut T) -> W>,
}
impl<T, W> BindIterator<T, W> {
    pub(crate) fn new(
        process: *mut T,
        get_next: Box<dyn Fn(*mut T) -> *mut T>,
        ctor: Box<dyn Fn(*mut T) -> W>,
    ) -> Self {
        Self {
            current: process,
            get_next,
            ctor,
        }
    }
}
impl<T, W> Iterator for BindIterator<T, W> {
    type Item = W;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        if current.is_null() {
            None
        } else {
            self.current = (self.get_next)(self.current);
            Some((self.ctor)(current))
        }
    }
}
