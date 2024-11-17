use crate::error::Error;
use druid::Data;

#[derive(Clone, Debug, Data)]
pub enum Promise<T: Data, D: Data = (), E: Data = Error> {
    Empty,
    Deferred { def: D },
    Resolved { def: D, val: T },
    Rejected { def: D, err: E },
}

impl<D: Data + Default, T: Data, E: Data> Promise<T, D, E> {
    pub fn defer_default(&mut self) {
        *self = Self::Deferred { def: D::default() }
    }

    pub fn resolve(&mut self, def: D, val: T) {
        *self = Self::Resolved { def, val };
    }

    pub fn reject(&mut self, def: D, err: E) {
        *self = Self::Rejected { def, err };
    }

    pub fn resolve_or_reject(&mut self, def: D, res: Result<T, E>) {
        match res {
            Ok(val) => self.resolve(def, val),
            Err(err) => self.reject(def, err),
        }
    }
}
