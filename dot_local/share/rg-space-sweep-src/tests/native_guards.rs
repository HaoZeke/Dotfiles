use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = env::temp_dir().join(format!(
            "space-sweep-native-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn meson(&self, build: &Path, source: &Path) {
        fs::create_dir_all(build.join("meson-private")).unwrap();
        fs::create_dir_all(build.join("meson-info")).unwrap();
        fs::create_dir_all(source).unwrap();
        fs::write(build.join("meson-private/coredata.dat"), b"generated").unwrap();
        fs::write(
            build.join("meson-info/meson-info.json"),
            serde_json::to_vec(
                &serde_json::json!({"directories": {"source": source, "build": build}}),
            )
            .unwrap(),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn all_includes_native_and_arbitrary_marker_names() {
    assert!(Category::expand("all").unwrap().contains(&Category::Native));
    assert!(walk_names_for(&[Category::Native]).contains(&"meson-private"));
    assert!(walk_names_for(&[Category::Native]).contains(&"CMakeFiles"));
    let fixture = Fixture::new();
    let build = fixture.0.join("unconventional-release-tree");
    fixture.meson(&build, &fixture.0.join("source"));
    assert!(has_native_build_markers(&build));
    assert!(validate_native_build(&build).is_ok());
}

#[test]
fn native_rejects_in_source_and_mismatched_metadata() {
    let fixture = Fixture::new();
    let build = fixture.0.join("build");
    fixture.meson(&build, &build);
    assert!(validate_native_build(&build).is_err());
    let source = fixture.0.join("source");
    fixture.meson(&build, &source);
    let other = fixture.0.join("other");
    fs::create_dir(&other).unwrap();
    fs::write(
        build.join("meson-info/meson-info.json"),
        serde_json::to_vec(&serde_json::json!({"directories": {"source": source, "build": other}}))
            .unwrap(),
    )
    .unwrap();
    assert!(validate_native_build(&build).is_err());
}

#[test]
fn marker_walk_enters_non_cargo_target_under_worktrees() {
    let fixture = Fixture::new();
    let build = fixture.0.join("wt/project/target");
    fixture.meson(&build, &fixture.0.join("wt/project/source"));
    let candidates = collect_candidates(&fixture.0, &Category::expand("all").unwrap()).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].path, build);
    assert_eq!(candidates[0].category, Category::Native);
}

#[test]
fn nested_candidates_count_once() {
    let root = PathBuf::from("/home/example/wt/project/build");
    let candidates = vec![
        Candidate {
            category: Category::Venv,
            path: root.join(".venv"),
            age_gated: true,
        },
        Candidate {
            category: Category::Native,
            path: root.clone(),
            age_gated: true,
        },
        Candidate {
            category: Category::Native,
            path: root.join("subproject/build"),
            age_gated: true,
        },
    ];
    let kept = collapse_native_descendants(candidates);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].path, root);
}

#[test]
fn failed_age_inspection_is_unknown_not_stale() {
    let fixture = Fixture::new();
    assert!(!is_stale(&fixture.0.join("missing"), 7, 1_700_000_000));
}

#[test]
fn nested_mounts_include_same_device_bind_mounts_and_kernel_escapes() {
    let row = |path: &str| format!("36 25 0:32 / {path} rw - tmpfs tmpfs rw\n");
    assert!(verify_no_nested_mounts(Path::new("/build"), &row("/build/bind")).is_err());
    assert!(verify_no_nested_mounts(Path::new("/build"), &row("/build-other")).is_ok());
    for (plain, encoded) in [
        ("/build a", "/build\\040a"),
        ("/build\ta", "/build\\011a"),
        ("/build\na", "/build\\012a"),
        ("/build\\a", "/build\\134a"),
    ] {
        assert!(
            verify_no_nested_mounts(Path::new(plain), &row(&format!("{encoded}/bind"))).is_err()
        );
    }
}

#[test]
fn malformed_mount_information_fails_closed() {
    for input in [
        "",
        "missing fields",
        "36 25 0:32 / relative rw - tmpfs tmpfs rw\n",
        "36 25 0:32 / /build\\999bad rw - tmpfs tmpfs rw\n",
    ] {
        assert!(verify_no_nested_mounts(Path::new("/build"), input).is_err());
    }
}
