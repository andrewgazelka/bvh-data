pub trait NonZeroArrayExt {
    fn max_index(&self) -> usize;
}

trait IsTrue {}
impl IsTrue for Assert<true> {}

struct Assert<const CHECK: bool>;

impl<const N: usize> NonZeroArrayExt for [u16; N]
where
    // todo: any *good* way to do this without a nightly feature?
    // todo v2: is this the best way to do this on nightly?
    Assert<{ N > 0 }>: IsTrue,
{
    // todo: bench... is this efficient?
    fn max_index(&self) -> usize {
        self.iter()
            .enumerate()
            .max_by_key(|(_, &v)| v)
            .map(|(i, _)| i)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_max_index() {
        let arr = [1, 2, 3, 4, 5];
        assert_eq!(arr.max_index(), 4);

        let arr = [1, 0, 8, 2, 3, 4, 5];
        assert_eq!(arr.max_index(), 2);
    }
}
