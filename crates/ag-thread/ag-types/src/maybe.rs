#[derive(Clone, PartialEq, Eq, Debug, Copy, Default)]
pub enum Maybe<T> {
  #[default]
  Absent,
  Present(T),
}
