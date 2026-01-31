/// Trait for matrix symmetry types.
///
/// Implemented by [`Symmetric`] and [`Asymmetric`] marker types.
///
pub trait Symmetry {}

pub struct Symmetric;
pub struct Asymmetric;

impl Symmetry for Symmetric {}
impl Symmetry for Asymmetric {}
