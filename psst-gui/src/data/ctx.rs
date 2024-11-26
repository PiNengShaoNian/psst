use druid::{lens::Field, Data, Lens, LensExt};

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
