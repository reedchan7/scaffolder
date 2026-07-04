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

fn run_scaffolder(
    dir: &std::path::Path,
    home: &std::path::Path,
    args: &[&str],
) -> assert_cmd::assert::Assert {
    Command::cargo_bin("scaffolder")
        .unwrap()
        .current_dir(dir)
        .env("HOME", home)
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
    assert!(!pkg.contains("\"prepare\""));
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
fn agent_trust_claude_creates_local_settings() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();

    run_scaffolder(&project, &home, &["agent", "trust", "claude"]).success();

    let settings = std::fs::read_to_string(project.join(".claude/settings.local.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&settings).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "permissions": {
                "defaultMode": "bypassPermissions"
            }
        })
    );
}

#[test]
fn agent_trust_claude_merges_existing_settings() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("project");
    std::fs::create_dir_all(project.join(".claude")).unwrap();
    std::fs::write(
        project.join(".claude/settings.local.json"),
        r#"{"env":{"FOO":"bar"},"permissions":{"allow":["Read"]}}"#,
    )
    .unwrap();

    run_scaffolder(&project, &home, &["agent", "trust", "claude"]).success();

    let settings = std::fs::read_to_string(project.join(".claude/settings.local.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&settings).unwrap();
    assert_eq!(value["env"]["FOO"], "bar");
    assert_eq!(value["permissions"]["allow"], serde_json::json!(["Read"]));
    assert_eq!(
        value["permissions"]["defaultMode"],
        serde_json::json!("bypassPermissions")
    );
}

#[test]
fn agent_trust_writes_supported_agent_configs_without_dropping_existing_data() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();

    std::fs::create_dir_all(home.join(".kimi-code")).unwrap();
    std::fs::write(
        home.join(".kimi-code/config.toml"),
        "default_model = \"kimi-code/kimi-for-coding\"\n",
    )
    .unwrap();

    std::fs::create_dir_all(home.join(".gemini/antigravity-cli")).unwrap();
    std::fs::write(
        home.join(".gemini/antigravity-cli/settings.json"),
        r#"{"model":"Gemini","trustedWorkspaces":["/already"]}"#,
    )
    .unwrap();

    std::fs::create_dir_all(home.join(".pi/agent")).unwrap();
    std::fs::write(home.join(".pi/agent/trust.json"), r#"{"/already":false}"#).unwrap();

    for agent in ["codex", "kimi-code", "agy", "reasonix", "pi"] {
        run_scaffolder(&project, &home, &["agent", "trust", agent]).success();
    }

    let codex_project = std::fs::read_to_string(project.join(".codex/config.toml")).unwrap();
    assert!(codex_project.contains("approval_policy = \"never\""));
    assert!(codex_project.contains("sandbox_mode = \"danger-full-access\""));

    let project_path = project.canonicalize().unwrap().display().to_string();
    let codex_user = std::fs::read_to_string(home.join(".codex/config.toml")).unwrap();
    assert!(codex_user.contains(&format!("[projects.\"{project_path}\"]")));
    assert!(codex_user.contains("trust_level = \"trusted\""));

    let kimi = std::fs::read_to_string(home.join(".kimi-code/config.toml")).unwrap();
    assert!(kimi.contains("default_model = \"kimi-code/kimi-for-coding\""));
    assert!(kimi.contains("default_permission_mode = \"yolo\""));

    let agy: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(home.join(".gemini/antigravity-cli/settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(agy["model"], "Gemini");
    assert_eq!(agy["toolPermission"], "always-proceed");
    assert!(
        agy["trustedWorkspaces"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(project_path))
    );

    let reasonix = std::fs::read_to_string(project.join("reasonix.toml")).unwrap();
    assert!(reasonix.contains("[permissions]"));
    assert!(reasonix.contains("mode = \"allow\""));

    let pi: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(home.join(".pi/agent/trust.json")).unwrap())
            .unwrap();
    assert_eq!(pi["/already"], serde_json::json!(false));
    assert_eq!(pi[project_path], serde_json::json!(true));
}

#[test]
fn agent_trust_accepts_multiple_agents_in_one_command() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let project = tmp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();

    run_scaffolder(
        &project,
        &home,
        &["agent", "trust", "claude", "reasonix", "pi"],
    )
    .success();

    assert!(project.join(".claude/settings.local.json").is_file());
    assert!(project.join("reasonix.toml").is_file());

    let project_path = project.canonicalize().unwrap().display().to_string();
    let pi: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(home.join(".pi/agent/trust.json")).unwrap())
            .unwrap();
    assert_eq!(pi[project_path], serde_json::json!(true));
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

#[test]
fn dir_flag_creates_in_given_directory() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--dir",
            "nested/sub",
        ],
    )
    .success();
    assert!(tmp.path().join("nested/sub/demo/package.json").is_file());
    // Nothing was created directly under the cwd.
    assert!(!tmp.path().join("demo").exists());
}

#[test]
fn bun_test_without_bun_pm_is_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--pm",
            "npm",
            "--test",
            "bun",
        ],
    )
    .failure()
    .stderr(predicate::str::contains("requires --pm bun"));
}

#[test]
fn bun_pm_defaults_to_bun_runtime_and_test_runner() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--pm",
            "bun",
        ],
    )
    .success();

    let root = tmp.path().join("demo");
    assert!(!root.join(".nvmrc").exists());
    assert!(!root.join("vitest.config.ts").exists());
    assert!(root.join("biome.json").is_file());
    assert!(!root.join("eslint.config.mjs").exists());
    assert!(!root.join(".prettierrc").exists());
    assert!(!root.join(".prettierignore").exists());

    let pkg: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("package.json")).unwrap()).unwrap();
    assert_eq!(pkg["packageManager"], serde_json::json!("bun@1.3.14"));
    assert!(pkg.get("engines").is_none());
    assert_eq!(
        pkg["scripts"]["dev"],
        serde_json::json!("bun --watch run src/index.ts")
    );
    assert_eq!(
        pkg["scripts"]["build"],
        serde_json::json!("bun build ./src/index.ts --outdir ./dist --target bun")
    );
    assert_eq!(
        pkg["scripts"]["start"],
        serde_json::json!("bun run src/index.ts")
    );
    assert_eq!(pkg["scripts"]["test"], serde_json::json!("bun test"));
    assert_eq!(
        pkg["scripts"]["typecheck"],
        serde_json::json!("bun run --bun tsc --noEmit")
    );
    assert_eq!(
        pkg["scripts"]["lint"],
        serde_json::json!("bun run --bun biome lint .")
    );
    assert_eq!(
        pkg["scripts"]["format"],
        serde_json::json!("bun run --bun biome format --write .")
    );
    assert_eq!(
        pkg["scripts"]["check"],
        serde_json::json!("bun run --bun biome check . && bun run --bun tsc --noEmit && bun test")
    );
    let scripts = pkg["scripts"].as_object().unwrap();
    assert!(
        scripts
            .values()
            .all(|script| !script.as_str().unwrap().contains("node"))
    );
    assert!(pkg["devDependencies"].get("@biomejs/biome").is_some());
    assert!(pkg["devDependencies"].get("@types/bun").is_some());
    assert!(pkg["devDependencies"].get("@eslint/js").is_none());
    assert!(pkg["devDependencies"].get("@types/node").is_none());
    assert!(pkg["devDependencies"].get("eslint").is_none());
    assert!(pkg["devDependencies"].get("prettier").is_none());
    assert!(pkg["devDependencies"].get("tsx").is_none());
    assert!(pkg["devDependencies"].get("typescript-eslint").is_none());
    assert!(pkg["devDependencies"].get("vitest").is_none());
    assert!(pkg["scripts"].get("prepare").is_none());

    let tsconfig = std::fs::read_to_string(root.join("tsconfig.json")).unwrap();
    assert!(tsconfig.contains(r#""types": ["bun"]"#));
    assert!(tsconfig.contains(r#""moduleResolution": "bundler""#));
    assert!(tsconfig.contains(r#""noEmit": true"#));
    assert!(!tsconfig.contains(r#""outDir""#));

    let test = std::fs::read_to_string(root.join("src/index.test.ts")).unwrap();
    assert!(test.contains(r#"from "bun:test""#));
    assert!(!test.contains("node:test"));
    assert!(!test.contains("vitest"));
}

#[test]
fn bun_pm_rejects_node_test_runner() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--pm",
            "bun",
            "--test",
            "node",
        ],
    )
    .failure()
    .stderr(predicate::str::contains("requires --test bun"));
}

#[test]
fn bun_pm_rejects_vitest_test_runner() {
    let tmp = tempfile::tempdir().unwrap();
    run_new(
        tmp.path(),
        &[
            "typescript-node",
            "demo",
            "--no-git",
            "--no-install",
            "--pm",
            "bun",
            "--test",
            "vitest",
        ],
    )
    .failure()
    .stderr(predicate::str::contains("requires --test bun"));
}
