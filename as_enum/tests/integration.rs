use as_enum::AsEnum;
use as_enum::AsEnum as AsEnumDerive;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, AsEnumDerive)]
pub enum Directedness {
    Directed,
    Undirected,
}

#[test]
fn enum_as_enum_returns_self() {
    assert_eq!(Directedness::Directed.as_enum(), Directedness::Directed);
    assert_eq!(Directedness::Undirected.as_enum(), Directedness::Undirected);
}

#[test]
fn struct_as_enum() {
    assert_eq!(Directed.as_enum(), Directedness::Directed);
    assert_eq!(Undirected.as_enum(), Directedness::Undirected);
}

#[test]
fn from_impls() {
    let d: Directedness = Directed.into();
    assert_eq!(d, Directedness::Directed);
    let u: Directedness = Undirected.into();
    assert_eq!(u, Directedness::Undirected);
}

#[test]
fn try_from_impls() {
    assert_eq!(Directed::try_from(Directedness::Directed), Ok(Directed));
    assert_eq!(Directed::try_from(Directedness::Undirected), Err(()));
    assert_eq!(
        Undirected::try_from(Directedness::Undirected),
        Ok(Undirected)
    );
    assert_eq!(Undirected::try_from(Directedness::Directed), Err(()));
}
