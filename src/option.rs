// TODO: use https://doc.rust-lang.org/std/option/enum.Option.html#method.contains
// TODO: when it's stabilized
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
