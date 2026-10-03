use super::{manifest, markdown, parquet};
use cogmax_domain::{
    memory::{Authority, Confidence, Memory, MemoryKind},
    scope::MemoryScope,
};

fn memory(content: &str) -> Memory {
    Memory::new(
        MemoryScope::new("user:alice").unwrap(),
        MemoryKind::Fact,
        content.into(),
        Confidence::High,
        Authority::Explicit,
    )
}

#[test]
fn markdown_and_manifest_are_deterministic() {
    let memories = vec![memory("beta"), memory("alpha")];
    let text = markdown(&memories);
    let same = markdown(&memories);
    assert_eq!(text, same);
    assert_eq!(
        manifest(&text, memories.len()),
        manifest(&same, memories.len())
    );
}

#[test]
fn parquet_has_parquet_magic_bytes() {
    let bytes = parquet(&[memory("alpha")]).unwrap();
    assert_eq!(&bytes[..4], b"PAR1");
    assert_eq!(&bytes[bytes.len() - 4..], b"PAR1");
}
