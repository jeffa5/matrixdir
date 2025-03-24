pub struct Read;
pub struct Write;

pub trait ReaderWriter {}

impl ReaderWriter for Read {}
impl ReaderWriter for Write {}
