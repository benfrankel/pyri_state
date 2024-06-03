use syn::{parse_str, Path, PathSegment};

pub(crate) fn concat(mut base_path: Path, suffix: &str) -> Path {
    let suffix = parse_str::<PathSegment>(suffix).unwrap();
    base_path.segments.push(suffix);
    base_path
}
