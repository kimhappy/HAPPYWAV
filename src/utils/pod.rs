pub trait Pod: Sized {}

impl Pod for u8    {}
impl Pod for u16   {}
impl Pod for u32   {}
impl Pod for u64   {}
impl Pod for u128  {}
impl Pod for usize {}
impl Pod for i8    {}
impl Pod for i16   {}
impl Pod for i32   {}
impl Pod for i64   {}
impl Pod for i128  {}
impl Pod for isize {}
impl Pod for f32   {}
impl Pod for f64   {}

impl< T: Pod, const N: usize > Pod for [T; N] {}
