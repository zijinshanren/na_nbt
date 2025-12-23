use crate::{ConfigMut, ValueBase};

pub trait ValueMut<'s>: ValueBase<ConfigRef = Self::ConfigMut> {
    type ConfigMut: ConfigMut;
}
