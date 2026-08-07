use std::ops::Deref;

pub trait FilterEmptyString<T: Deref<Target=str>>
{
    fn empty_as_none(self) -> Option<T>;
}

impl<T> FilterEmptyString<T> for Option<T>
    where
        T: Deref<Target=str>,
{
    fn empty_as_none(self) -> Option<T> {
        match &self {
            Some(s) if s.deref().trim().is_empty() => None,
            _ => self,
        }
    }
}