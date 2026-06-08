use serde_json::{Map, Value, json};

use crate::config::{PackageManager, ScaffoldConfig, TestFramework};

/// Normalize and validate a project/package name (npm rules, path-safe).
pub fn normalize_package_name(input: &str) -> anyhow::Result<String> {
    let name = input.trim().to_lowercase().replace(' ', "-");
    let valid = !name.is_empty()
        && name.len() <= 214
        && !name.starts_with(['.', '_'])
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '.' | '_'));
    if !valid {
        anyhow::bail!(
            "invalid package name: {input:?} (use lowercase letters, digits, '-', '.', '_')"
        );
    }
    // npm-reserved / unsafe-as-directory names.
    const RESERVED: [&str; 3] = ["node_modules", "favicon.ico", "."];
    if RESERVED.contains(&name.as_str()) {
        anyhow::bail!("reserved package name: {name:?}");
    }
    Ok(name)
}

/// Accept only a bare major version integer in a sane range.
pub fn parse_node(input: &str) -> anyhow::Result<u32> {
    let n: u32 = input.trim().parse().map_err(|_| {
        anyhow::anyhow!("--node must be a major version integer like 24, got {input:?}")
    })?;
    if !(18..=99).contains(&n) {
        anyhow::bail!("--node out of supported range (18-99): {n}");
    }
    Ok(n)
}

pub struct RenderCtx {
    pub name: String,
    pub node: u32,
    pub pm: String,
    pub import_ext: String,
    pub year: i32,
}

impl ScaffoldConfig {
    pub fn render_ctx(&self) -> RenderCtx {
        RenderCtx {
            name: self.name.clone(),
            node: self.node,
            pm: self.pm.bin().to_string(),
            import_ext: self.module.import_ext().to_string(),
            year: current_year(),
        }
    }
}

/// Replace `{{token}}` placeholders. Unknown tokens are left untouched.
pub fn render(content: &str, ctx: &RenderCtx) -> String {
    content
        .replace("{{name}}", &ctx.name)
        .replace("{{node}}", &ctx.node.to_string())
        .replace("{{pm}}", &ctx.pm)
        .replace("{{import_ext}}", &ctx.import_ext)
        .replace("{{year}}", &ctx.year.to_string())
}

/// Current Gregorian year from the system clock, no external date crate.
pub fn current_year() -> i32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut days = (secs / 86_400) as i64;
    let mut year: i64 = 1970;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let year_days = if leap { 366 } else { 365 };
        if days < year_days {
            break;
        }
        days -= year_days;
        year += 1;
    }
    year as i32
}

/// Build a deterministic, ordered `package.json` as a pretty string (trailing newline).
pub fn build_package_json(cfg: &ScaffoldConfig) -> String {
    let mut root = Map::new();
    root.insert("name".into(), json!(cfg.name));
    root.insert("version".into(), json!("0.1.0"));
    root.insert("type".into(), json!(cfg.module.package_type()));
    match cfg.license {
        Some(l) => {
            root.insert("license".into(), json!(l.spdx()));
        }
        None => {
            root.insert("private".into(), json!(true));
        }
    }
    root.insert(
        "packageManager".into(),
        json!(cfg.pm.package_manager_field()),
    );
    root.insert(
        "engines".into(),
        json!({ "node": format!(">={}", cfg.node) }),
    );
    root.insert("main".into(), json!("dist/index.js"));

    let mut scripts = Map::new();
    // tsx runs TS directly for both esm and cjs and across Node versions, unlike
    // `node --watch src/index.ts` which breaks under type=commonjs and old Node.
    scripts.insert("dev".into(), json!("tsx watch src/index.ts"));
    scripts.insert("build".into(), json!("tsc"));
    scripts.insert("start".into(), json!("node dist/index.js"));
    scripts.insert("test".into(), json!(cfg.test.test_script()));
    scripts.insert("lint".into(), json!("eslint ."));
    scripts.insert("format".into(), json!("prettier --write ."));
    scripts.insert(
        "check".into(),
        json!(format!(
            "prettier --check . && eslint . && {}",
            cfg.test.test_script()
        )),
    );
    // `|| true` so a missing git repo (e.g. --no-git, or CI installs) never
    // fails the whole install; hooks still install normally when git is present.
    scripts.insert("prepare".into(), json!("lefthook install || true"));
    root.insert("scripts".into(), Value::Object(scripts));

    let mut dev = Map::new();
    dev.insert("@eslint/js".into(), json!("^10.0.1"));
    dev.insert("@types/node".into(), json!(format!("^{}", cfg.node)));
    dev.insert("eslint".into(), json!("^10.4.1"));
    dev.insert("globals".into(), json!("^17.6.0"));
    dev.insert("lefthook".into(), json!("^2.1.9"));
    dev.insert("prettier".into(), json!("^3.8.3"));
    dev.insert("tsx".into(), json!("^4.22.4"));
    dev.insert("typescript".into(), json!("^6.0.3"));
    dev.insert("typescript-eslint".into(), json!("^8.60.1"));
    if cfg.test == TestFramework::Vitest {
        dev.insert("vitest".into(), json!("^4.1.8"));
    }
    root.insert("devDependencies".into(), Value::Object(dev));

    // Note: pnpm's build-script allowlist lives in pnpm-workspace.yaml
    // (`allowBuilds`), not in this file — pnpm 10+ no longer reads a `pnpm`
    // field here. See selected_files / the pnpm-workspace.yaml template.

    let mut out = serde_json::to_string_pretty(&Value::Object(root))
        .expect("package.json serialization is infallible");
    out.push('\n');
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSpec {
    /// Path inside the embedded template folder.
    pub src: &'static str,
    /// Relative path written into the generated project.
    pub dest: &'static str,
}

/// Which embedded files to emit, given the chosen options.
/// (package.json and AI files are handled separately in scaffold.rs.)
pub fn selected_files(cfg: &ScaffoldConfig) -> Vec<FileSpec> {
    let mut v = vec![
        FileSpec {
            src: "tsconfig.json",
            dest: "tsconfig.json",
        },
        FileSpec {
            src: "index.ts",
            dest: "src/index.ts",
        },
        FileSpec {
            src: "eslint.config.mjs",
            dest: "eslint.config.mjs",
        },
        FileSpec {
            src: "prettierrc",
            dest: ".prettierrc",
        },
        FileSpec {
            src: "prettierignore",
            dest: ".prettierignore",
        },
        FileSpec {
            src: "Makefile",
            dest: "Makefile",
        },
        FileSpec {
            src: "lefthook.yml",
            dest: "lefthook.yml",
        },
        FileSpec {
            src: "gitignore",
            dest: ".gitignore",
        },
        FileSpec {
            src: "nvmrc",
            dest: ".nvmrc",
        },
        FileSpec {
            src: "README.md",
            dest: "README.md",
        },
    ];
    match cfg.test {
        TestFramework::Vitest => {
            v.push(FileSpec {
                src: "vitest.config.ts",
                dest: "vitest.config.ts",
            });
            v.push(FileSpec {
                src: "index.vitest.test.ts",
                dest: "src/index.test.ts",
            });
        }
        TestFramework::Node => {
            v.push(FileSpec {
                src: "index.node.test.ts",
                dest: "src/index.test.ts",
            });
        }
    }
    if let Some(l) = cfg.license {
        v.push(FileSpec {
            src: l.template_file(),
            dest: "LICENSE",
        });
    }
    // pnpm 10+ requires build scripts (e.g. lefthook's postinstall) to be
    // allowlisted in pnpm-workspace.yaml, or `pnpm install` exits non-zero.
    if cfg.pm == PackageManager::Pnpm {
        v.push(FileSpec {
            src: "pnpm-workspace.yaml",
            dest: "pnpm-workspace.yaml",
        });
    }
    v
}

#[cfg(test)]
mod name_tests {
    use super::*;

    #[test]
    fn accepts_simple_name() {
        assert_eq!(normalize_package_name("my-app").unwrap(), "my-app");
    }

    #[test]
    fn lowercases_and_replaces_spaces() {
        assert_eq!(normalize_package_name("My App").unwrap(), "my-app");
    }

    #[test]
    fn rejects_empty() {
        assert!(normalize_package_name("   ").is_err());
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(normalize_package_name("../evil").is_err());
        assert!(normalize_package_name("a/b").is_err());
    }

    #[test]
    fn rejects_leading_dot_or_underscore() {
        assert!(normalize_package_name(".hidden").is_err());
        assert!(normalize_package_name("_x").is_err());
    }

    #[test]
    fn rejects_reserved_names() {
        assert!(normalize_package_name("node_modules").is_err());
        assert!(normalize_package_name("favicon.ico").is_err());
    }
}

#[cfg(test)]
mod node_tests {
    use super::*;

    #[test]
    fn accepts_major_integer() {
        assert_eq!(parse_node("24").unwrap(), 24);
        assert_eq!(parse_node("22").unwrap(), 22);
    }

    #[test]
    fn rejects_ranges_and_dotted() {
        assert!(parse_node(">=24").is_err());
        assert!(parse_node("lts/*").is_err());
        assert!(parse_node("24.1.0").is_err());
        assert!(parse_node("").is_err());
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(parse_node("3").is_err());
        assert!(parse_node("999").is_err());
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    fn ctx() -> RenderCtx {
        RenderCtx {
            name: "demo".into(),
            node: 24,
            pm: "pnpm".into(),
            import_ext: ".js".into(),
            year: 2026,
        }
    }

    #[test]
    fn replaces_all_placeholders() {
        let out = render(
            "a {{name}} b {{node}} c {{pm}} e {{import_ext}} f {{year}}",
            &ctx(),
        );
        assert_eq!(out, "a demo b 24 c pnpm e .js f 2026");
    }

    #[test]
    fn leaves_unknown_tokens() {
        assert_eq!(render("{{unknown}}", &ctx()), "{{unknown}}");
    }

    #[test]
    fn current_year_is_reasonable() {
        let y = current_year();
        assert!(y >= 2026 && y < 2100, "got {y}");
    }
}

#[cfg(test)]
mod pkg_tests {
    use super::*;
    use crate::config::*;

    fn base() -> ScaffoldConfig {
        ScaffoldConfig {
            name: "demo".into(),
            pm: PackageManager::Pnpm,
            test: TestFramework::Vitest,
            module: ModuleSystem::Esm,
            node: 24,
            license: None,
            ai: false,
            git: true,
            install: true,
        }
    }

    fn json(cfg: &ScaffoldConfig) -> serde_json::Value {
        serde_json::from_str(&build_package_json(cfg)).unwrap()
    }

    #[test]
    fn private_when_no_license() {
        let v = json(&base());
        assert_eq!(v["private"], serde_json::json!(true));
        assert!(v.get("license").is_none());
    }

    #[test]
    fn license_removes_private() {
        let mut c = base();
        c.license = Some(License::Mit);
        let v = json(&c);
        assert_eq!(v["license"], serde_json::json!("MIT"));
        assert!(v.get("private").is_none());
    }

    #[test]
    fn module_type_and_engines_and_pm() {
        let v = json(&base());
        assert_eq!(v["type"], serde_json::json!("module"));
        assert_eq!(v["engines"]["node"], serde_json::json!(">=24"));
        assert_eq!(v["packageManager"], serde_json::json!("pnpm@11.5.2"));
        assert_eq!(
            v["devDependencies"]["@types/node"],
            serde_json::json!("^24")
        );
    }

    #[test]
    fn cjs_type() {
        let mut c = base();
        c.module = ModuleSystem::Cjs;
        assert_eq!(json(&c)["type"], serde_json::json!("commonjs"));
    }

    #[test]
    fn vitest_dep_only_for_vitest() {
        let v = json(&base());
        assert!(v["devDependencies"].get("vitest").is_some());
        assert_eq!(v["scripts"]["test"], serde_json::json!("vitest run"));

        let mut c = base();
        c.test = TestFramework::Node;
        let v2 = json(&c);
        assert!(v2["devDependencies"].get("vitest").is_none());
        assert_eq!(
            v2["scripts"]["test"],
            serde_json::json!("tsc && node --test \"dist/**/*.test.js\"")
        );
    }

    #[test]
    fn has_prepare_lefthook() {
        assert_eq!(
            json(&base())["scripts"]["prepare"],
            serde_json::json!("lefthook install || true")
        );
    }
}

#[cfg(test)]
mod select_tests {
    use super::*;
    use crate::config::*;

    fn base() -> ScaffoldConfig {
        ScaffoldConfig {
            name: "demo".into(),
            pm: PackageManager::Pnpm,
            test: TestFramework::Vitest,
            module: ModuleSystem::Esm,
            node: 24,
            license: None,
            ai: false,
            git: true,
            install: true,
        }
    }

    fn dests(cfg: &ScaffoldConfig) -> Vec<&'static str> {
        selected_files(cfg).into_iter().map(|f| f.dest).collect()
    }

    #[test]
    fn always_includes_baseline() {
        let d = dests(&base());
        for f in [
            "tsconfig.json",
            "src/index.ts",
            "eslint.config.mjs",
            ".prettierrc",
            ".prettierignore",
            "Makefile",
            "lefthook.yml",
            ".gitignore",
            ".nvmrc",
            "README.md",
        ] {
            assert!(d.contains(&f), "missing {f}");
        }
    }

    #[test]
    fn vitest_includes_config_and_test() {
        let d = dests(&base());
        assert!(d.contains(&"vitest.config.ts"));
        assert!(d.contains(&"src/index.test.ts"));
    }

    #[test]
    fn node_excludes_vitest_config() {
        let mut c = base();
        c.test = TestFramework::Node;
        let d = dests(&c);
        assert!(!d.contains(&"vitest.config.ts"));
        assert!(d.contains(&"src/index.test.ts"));
    }

    #[test]
    fn license_adds_license_file() {
        let mut c = base();
        c.license = Some(License::Apache2);
        let files = selected_files(&c);
        let lic = files.iter().find(|f| f.dest == "LICENSE").unwrap();
        assert_eq!(lic.src, "license-apache-2.0.txt");
    }

    #[test]
    fn no_license_no_license_file() {
        assert!(!dests(&base()).contains(&"LICENSE"));
    }

    #[test]
    fn pnpm_includes_workspace_yaml() {
        assert!(dests(&base()).contains(&"pnpm-workspace.yaml"));
    }

    #[test]
    fn non_pnpm_excludes_workspace_yaml() {
        let mut c = base();
        c.pm = PackageManager::Npm;
        assert!(!dests(&c).contains(&"pnpm-workspace.yaml"));
    }
}
