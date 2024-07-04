use syn::{parse_str, Path, PathSegment};

pub(crate) fn concat(base_path: &Path, suffix: &str) -> Path {
    let mut base_path = base_path.clone();
    let suffix = parse_str::<PathSegment>(suffix).unwrap();
    base_path.segments.push(suffix);
    base_path
}
