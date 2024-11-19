use crate::error::Error;
use druid::Data;

#[derive(Clone, Debug, Data)]
pub enum Promise<T: Data, D: Data = (), E: Data = Error> {
    Empty,
    Deferred { def: D },
    Resolved { def: D, val: T },
    Rejected { def: D, err: E },
}

#[derive(Eq, PartialEq, Debug)]
pub enum PromiseState {
    Empty,
    Deferred,
    Resolved,
    Rejected,
}

impl<T: Data, D: Data, E: Data> Promise<T, D, E> {
    pub fn state(&self) -> PromiseState {
        match self {
            Self::Empty => PromiseState::Empty,
            Self::Deferred { .. } => PromiseState::Deferred,
            Self::Resolved { .. } => PromiseState::Resolved,
            Self::Rejected { .. } => PromiseState::Rejected,
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.state() == PromiseState::Resolved
    }

    pub fn is_deferred(&self, d: &D) -> bool
    where
        D: PartialEq,
    {
        matches!(self, Self::Deferred { def } if def == d)
    }
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
