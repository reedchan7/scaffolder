use assert_cmd::Command;
use predicates::prelude::*;

fn run_new(dir: &std::path::Path, args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("scaffolder")
        .unwrap()
        .current_dir(dir)
        .arg("new")
        .args(args)
        .assert()
}

#[test]
fn generates_default_project() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &["typescript-node", "demo", "--no-git", "--no-install"],
    )
    .success();

    let root = tmp.path().join("demo");
    for f in [
        "package.json",
        "tsconfig.json",
        "src/index.ts",
        "src/index.test.ts",
        "eslint.config.mjs",
        ".prettierrc",
        "Makefile",
        "lefthook.yml",
        "vitest.config.ts",
        ".gitignore",
        ".nvmrc",
        "README.md",
    ] {
        assert!(root.join(f).is_file(), "missing {f}");
    }
    let pkg = std::fs::read_to_string(root.join("package.json")).unwrap();
    assert!(pkg.contains("\"private\": true"));
    assert!(pkg.contains("\"type\": \"module\""));
    assert!(pkg.contains("\"packageManager\": \"pnpm@11.5.2\""));
    assert!(!root.join("CLAUDE.md").exists());
    assert!(!root.join("LICENSE").exists());
}

#[test]
fn license_mit_drops_private_and_adds_file() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--license",
            "MIT",
        ],
    )
    .success();
    let root = tmp.path().join("demo");
    let pkg = std::fs::read_to_string(root.join("package.json")).unwrap();
    assert!(pkg.contains("\"license\": \"MIT\""));
    assert!(!pkg.contains("\"private\""));
    assert!(root.join("LICENSE").is_file());
}

#[test]
fn test_node_has_no_vitest() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--test",
            "node",
        ],
    )
    .success();
    let root = tmp.path().join("demo");
    assert!(!root.join("vitest.config.ts").exists());
    let test = std::fs::read_to_string(root.join("src/index.test.ts")).unwrap();
    assert!(test.contains("node:test"));
    let pkg = std::fs::read_to_string(root.join("package.json")).unwrap();
    assert!(!pkg.contains("vitest"));
}

#[test]
fn module_cjs_sets_commonjs_and_extensionless_import() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--module",
            "cjs",
        ],
    )
    .success();
    let root = tmp.path().join("demo");
    let pkg = std::fs::read_to_string(root.join("package.json")).unwrap();
    assert!(pkg.contains("\"type\": \"commonjs\""));
    let test = std::fs::read_to_string(root.join("src/index.test.ts")).unwrap();
    assert!(
        test.contains("\"./index\""),
        "cjs import should be extensionless"
    );
}

#[test]
fn ai_flag_generates_guidelines() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--ai",
        ],
    )
    .success();
    let root = tmp.path().join("demo");
    assert!(root.join("CLAUDE.md").is_file());
    assert!(root.join("AGENTS.md").is_file());
}

#[test]
fn refuses_non_empty_target() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("demo")).unwrap();
    std::fs::write(tmp.path().join("demo/x"), "hi").unwrap();
    run_new(
        tmp.path(),
        &["typescript-node", "demo", "--no-git", "--no-install"],
    )
    .failure()
    .stderr(predicate::str::contains("not empty"));
}

#[test]
fn rejects_bad_node_version() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--node",
            ">=24",
        ],
    )
    .failure();
}

#[test]
fn single_positional_is_treated_as_name() {
    let tmp = tempfile::tempdir().unwrap();
    // `scaffolder new my-app` (one positional) must create ./my-app, not error
    // out as an unknown template.
    run_new(tmp.path(), &["my-app", "--no-git", "--no-install"]).success();
    let pkg = std::fs::read_to_string(tmp.path().join("my-app/package.json")).unwrap();
    assert!(pkg.contains("\"name\": \"my-app\""));
}

#[test]
fn dev_script_uses_tsx() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &["typescript-node", "demo", "--no-git", "--no-install"],
    )
    .success();
    let pkg = std::fs::read_to_string(tmp.path().join("demo/package.json")).unwrap();
    assert!(pkg.contains("\"dev\": \"tsx watch src/index.ts\""));
    assert!(pkg.contains("\"tsx\":"));
}

#[test]
fn list_prints_template() {
    Command::cargo_bin("scaffolder")
        .unwrap()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("typescript-node"));
}
