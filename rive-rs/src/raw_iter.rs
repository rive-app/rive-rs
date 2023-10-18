use core::{marker::PhantomData, mem};

pub trait Raw: Copy {
    type Item<'r>
    where
        Self: 'r;

    fn len(self) -> usize;
    unsafe fn get<'r>(self, index: usize) -> Self::Item<'r>;
}

#[derive(Clone, Debug)]
pub struct RawIter<'r, R: Raw> {
    raw: R,
    start: usize,
    end: usize,
    _phantom: PhantomData<&'r ()>,
}

impl<'r, R: Raw> RawIter<'r, R> {
    pub fn new(raw: R) -> Self {
        let end = raw.len();
        Self {
            raw,
            start: 0,
            end,
            _phantom: PhantomData,
        }
    }

    pub fn next(&mut self) -> Option<R::Item<'r>> {
        (self.start < self.end).then(|| {
            let start = self.start;
            let index = mem::replace(&mut self.start, start + 1);

            unsafe { self.raw.get(index) }
        })
    }

    pub fn nth(&mut self, n: usize) -> Option<R::Item<'r>> {
        let index = self.start + n;
        (index < self.end).then(|| unsafe { self.raw.get(index) })
    }

    pub fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }

    pub fn next_back(&mut self) -> Option<R::Item<'r>> {
        (self.start < self.end).then(|| {
            self.end -= 1;

            unsafe { self.raw.get(self.end) }
        })
    }

    pub fn nth_back(&mut self, n: usize) -> Option<R::Item<'r>> {
        let index = self.end.checked_sub(n + 1)?;
        (self.start <= index).then(|| unsafe { self.raw.get(index) })
    }
}

macro_rules! impl_iter {
    ( @raw_iter_type $raw:ty, $lt:lifetime ) => (crate::raw_iter::RawIter<$lt, $raw>);

    ( @raw_iter_type $raw:ty ) => (crate::raw_iter::RawIter<'static, $raw>);

    ( $iter:ident, $item:ident, $raw:ty $( , $lt:lifetime )? ) => {
        #[derive(Clone, Debug)]
        pub struct $iter$(<$lt>)?(impl_iter!(@raw_iter_type $raw $(, $lt)?));

        impl$(<$lt>)? $iter$(<$lt>)? {
            pub(crate) fn new(raw: $raw) -> Self {
                Self(crate::raw_iter::RawIter::new(raw))
            }
        }

        impl$(<$lt>)? Iterator for $iter$(<$lt>)? {
            type Item = $item$(<$lt>)?;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }

            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                self.0.nth(n)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.0.size_hint()
            }
        }

        impl$(<$lt>)? DoubleEndedIterator for $iter$(<$lt>)? {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.0.next_back()
            }

            fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                self.0.nth_back(n)
            }
        }

        impl$(<$lt>)? ExactSizeIterator for $iter$(<$lt>)? {}
    };
}

pub(crate) use impl_iter;
