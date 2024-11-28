use druid::{
    lens::{Field, Map},
    Data, Lens, LensExt,
};

use super::Promise;

#[derive(Clone, Data)]
pub struct Ctx<C, T> {
    pub ctx: C,
    pub data: T,
}

impl<C, T> Ctx<C, T>
where
    C: Data,
    T: Data,
{
    pub fn new(c: C, t: T) -> Self {
        Self { ctx: c, data: t }
    }

    pub fn make<S: Data>(cl: impl Lens<S, C>, tl: impl Lens<S, T>) -> impl Lens<S, Self> {
        CtxMake { cl, tl }
    }

    pub fn data() -> impl Lens<Self, T> {
        Field::new(|c: &Self| &c.data, |c: &mut Self| &mut c.data)
    }
}

struct CtxMake<CL, TL> {
    cl: CL,
    tl: TL,
}

impl<C, T, S, CL, TL> Lens<S, Ctx<C, T>> for CtxMake<CL, TL>
where
    C: Data,
    T: Data,
    S: Data,
    CL: Lens<S, C>,
    TL: Lens<S, T>,
{
    fn with<V, F: FnOnce(&Ctx<C, T>) -> V>(&self, data: &S, f: F) -> V
    where
        F: FnOnce(&Ctx<C, T>) -> V,
    {
        let c = self.cl.get(data);
        let t = self.tl.get(data);
        let ct = Ctx::new(c, t);
        f(&ct)
    }

    fn with_mut<V, F: FnOnce(&mut Ctx<C, T>) -> V>(&self, data: &mut S, f: F) -> V
    where
        F: FnOnce(&mut Ctx<C, T>) -> V,
    {
        let c = self.cl.get(data);
        let t = self.tl.get(data);
        let mut ct = Ctx::new(c, t);
        let v = f(&mut ct);
        self.cl.put(data, ct.ctx);
        self.tl.put(data, ct.data);
        v
    }
}

impl<C, PT, PD, PE> Ctx<C, Promise<PT, PD, PE>>
where
    C: Data,
    PT: Data,
    PD: Data,
    PE: Data,
{
    pub fn in_promise() -> impl Lens<Self, Promise<Ctx<C, PT>, PD, PE>> {
        Map::new(
            |c: &Self| match &c.data {
                Promise::Empty => Promise::Empty,
                Promise::Deferred { def } => Promise::Deferred {
                    def: def.to_owned(),
                },
                Promise::Resolved { def, val } => Promise::Resolved {
                    def: def.to_owned(),
                    val: Ctx::new(c.ctx.to_owned(), val.to_owned()),
                },
                Promise::Rejected { def, err } => Promise::Rejected {
                    def: def.to_owned(),
                    err: err.to_owned(),
                },
            },
            |c: &mut Self, p: Promise<Ctx<C, PT>, PD, PE>| match p {
                Promise::Empty => {
                    c.data = Promise::Empty;
                }
                Promise::Deferred { def } => {
                    c.data = Promise::Deferred { def };
                }
                Promise::Resolved { def, val } => {
                    c.data = Promise::Resolved { def, val: val.data };
                    c.ctx = val.ctx;
                }
                Promise::Rejected { def, err } => {
                    c.data = Promise::Rejected { def, err };
                }
            },
        )
    }
}
