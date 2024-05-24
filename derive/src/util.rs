use syn::{Path, PathSegment};

pub(crate) fn concat(mut base_path: Path, suffix: impl Into<PathSegment>) -> Path {
    base_path.segments.push(suffix.into());
    base_path
}
