use druid::im::Vector;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Page<T: Clone> {
    pub items: Vector<T>,
    pub limit: usize,
    pub offset: usize,
    pub total: usize,
}
