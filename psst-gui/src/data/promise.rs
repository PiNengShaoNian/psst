use crate::error::Error;
use druid::Data;

#[derive(Clone, Debug, Data)]
pub enum Promise<T: Data, D: Data = (), E: Data = Error> {
    Empty,
    Deferred { def: D },
    Resolved { def: D, val: T },
    Reject { def: D, err: E },
}

impl<D: Data + Default, T: Data, E: Data> Promise<T, D, E> {
    pub fn defer_default(&mut self) {
        *self = Self::Deferred { def: D::default() }
    }
}
