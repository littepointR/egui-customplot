use std::{fs, path::Path};

#[test]
fn source_domains_are_explicit_and_files_remain_focused() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for domain in [
        "axis",
        "data",
        "interaction",
        "layout",
        "model",
        "render",
        "widget",
    ] {
        assert!(
            src.join(domain).join("mod.rs").is_file(),
            "missing {domain} entry point"
        );
    }
    for entry in fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            assert!(
                matches!(
                    path.file_name().unwrap().to_str().unwrap(),
                    "lib.rs" | "diagnostics.rs"
                ),
                "organize new responsibilities in a domain directory: {}",
                path.display()
            );
        }
    }
    check_file_sizes(&src);
}

fn check_file_sizes(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_file_sizes(&path);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let lines = fs::read_to_string(&path).unwrap().lines().count();
            assert!(
                lines <= 400,
                "{} has {lines} lines; split by responsibility (see docs/ARCHITECTURE.md)",
                path.display()
            );
        }
    }
}
