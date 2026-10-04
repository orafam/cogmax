use super::{manifest, markdown, parquet, verify_manifest, ExportError};
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

#[test]
fn manifest_rejects_corrupted_snapshot() {
    let text = markdown(&[memory("alpha")]);
    let expected = manifest(&text, 1);
    assert!(verify_manifest(&text, &expected).is_ok());
    assert!(matches!(
        verify_manifest(&text.replace("alpha", "tampered"), &expected),
        Err(ExportError::ManifestMismatch)
    ));
}

#[test]
fn parquet_is_deterministic_independent_of_input_order() {
    let alpha = memory("alpha");
    let beta = memory("beta");
    assert_eq!(
        parquet(&[alpha.clone(), beta.clone()]).unwrap(),
        parquet(&[beta, alpha]).unwrap()
    );
}
