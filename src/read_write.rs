#[derive(Debug)]
pub struct Read;
#[derive(Debug)]
pub struct Write;

pub trait ReaderWriter {}

impl ReaderWriter for Read {}
impl ReaderWriter for Write {}
