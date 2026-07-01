//! End-to-end cross-language test: invokes the Python reference at
//! `scripts/verify_payload_hash.py` and asserts that the Rust
//! `compute_payload_hash` helper produces the same SHA-256 hex digests.
//!
//! This is the strongest available evidence that ADR-0016 / TZ 3.3
//! «формат должен совпадать с SyncServer» is satisfied: it runs an
//! independent Python interpreter and compares results byte-for-byte.
//!
//! The test is gated on `python3` being available on `PATH`. CI without
//! Python is rare in this project (Django tests already require it), but
//! the gate keeps the test from spuriously failing in unusual environments.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use warehouse_core::operations::outbox_service::compute_payload_hash;

fn python_hash_table() -> Vec<(String, String)> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("scripts/verify_payload_hash.py");
    assert!(
        script.exists(),
        "missing reference script at {} — was the workspace layout changed?",
        script.display(),
    );

    let output = Command::new("python3")
        .arg("--")
        .arg(script)
        .arg("--json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn python3");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("verify_payload_hash.py failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l)
                .unwrap_or_else(|e| panic!("invalid json from script: {e}: {l}"));
            let label = v["label"].as_str().unwrap().to_string();
            let digest = v["sha256"].as_str().unwrap().to_string();
            (label, digest)
        })
        .collect()
}

#[test]
fn compute_payload_hash_matches_python_reference() {
    // Skip the test silently when Python is not available, so this test
    // doesn't block builds in environments without a Python interpreter
    // (rare in this repo — Django tests require Python — but possible).
    let probe = Command::new("python3").arg("--version").output();
    if probe.is_err() {
        eprintln!("SKIP: python3 not found on PATH");
        return;
    }

    let table = python_hash_table();
    assert!(!table.is_empty(), "verify_payload_hash.py produced no rows");

    // Run a representative subset: ASCII baseline, Cyrillic name, mixed
    // Cyrillic/ASCII, non-BMP emoji, control chars, and the JSON-special
    // escapes row. The unit test in outbox_service.rs already covers the
    // complete table; this end-to-end check proves the script is wired
    // up and that the Rust↔Python byte sequences agree without the unit
    // test as a middleman.
    let cases: &[(&str, &str)] = &[
        // (label, input_json)
        ("ascii_object", r#"{"a":1,"b":[1,2,3]}"#),
        ("cyrillic_name", r#"{"name":"Склад"}"#),
        (
            "cyrillic_with_ascii",
            r#"{"comment":"Принято: 10 шт.","qty":5}"#,
        ),
        ("emoji_surrogate", r#"{"icon":"📦"}"#),
        ("control_chars", r#"{"text":"line1\nline2\ttab"}"#),
        ("json_specials", r#"{"text":"a\"b\\c"}"#),
    ];

    let table_ref: std::collections::HashMap<&str, &str> = table
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    for (label, input) in cases {
        let expected = table_ref
            .get(label)
            .unwrap_or_else(|| panic!("missing row in script output: {label}"));
        let actual = compute_payload_hash(input);
        assert_eq!(
            &actual, expected,
            "drift on fixture '{label}': expected {expected}, got {actual}\ninput: {input}",
        );
    }
}

// Suppress dead_code for unused helper kept for future manual inspection.
#[allow(dead_code)]
fn _write_to_stderr(s: &str) {
    let _ = std::io::stderr().write_all(s.as_bytes());
}
