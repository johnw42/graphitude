/// The result of mapping some set of items to a different set of the same type.
/// Used, for example, in the callbacks for graph compaction.
pub enum MappingResult<T> {
    /// The first item in the old set was remapped to the second item in the new set.
    Remapped(T, T),
    /// The item in the old set appears unchanged in the new set.
    Unchanged(T),
    /// The item was present in the old set but has been deleted in the new set.
    Deleted(T),
}
