pub trait Instantiate: Sized {
    type From;

    fn instantiate(from: &Self::From, index: Option<usize>) -> Option<Self>;
}
