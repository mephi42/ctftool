/* Stolen from unstable. */
pub fn contains<T, U>(x: &Option<T>, y: &U) -> bool
where
    U: PartialEq<T>,
{
    match x {
        Some(x) => y == x,
        None => false,
    }
}
