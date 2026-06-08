use std::path::Path;

use anyhow::Context;

use crate::config::ScaffoldConfig;
use crate::render::{build_package_json, render, selected_files};
use crate::templates;

/// Render and write the full project tree into `target`.
pub fn scaffold(cfg: &ScaffoldConfig, target: &Path) -> anyhow::Result<()> {
    if target.exists() {
        let mut entries = std::fs::read_dir(target)
            .with_context(|| format!("reading target dir {}", target.display()))?;
        if entries.next().is_some() {
            anyhow::bail!("target directory {} is not empty", target.display());
        }
    } else {
        std::fs::create_dir_all(target)
            .with_context(|| format!("creating target dir {}", target.display()))?;
    }

    let ctx = cfg.render_ctx();
    for spec in selected_files(cfg) {
        let raw = templates::get(spec.src)?;
        write_file(target, spec.dest, &render(&raw, &ctx))?;
    }

    write_file(target, "package.json", &build_package_json(cfg))?;

    if cfg.ai {
        write_file(target, "CLAUDE.md", templates::CLAUDE_MD)?;
        write_file(target, "AGENTS.md", templates::AGENTS_MD)?;
    }

    Ok(())
}

fn write_file(root: &Path, rel: &str, content: &str) -> anyhow::Result<()> {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&path, content).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    fn base(name: &str) -> ScaffoldConfig {
        ScaffoldConfig {
            name: name.into(),
            pm: PackageManager::Pnpm,
            test: TestFramework::Vitest,
            module: ModuleSystem::Esm,
            node: 24,
            license: None,
            ai: false,
            git: false,
            install: false,
        }
    }

    #[test]
    fn writes_baseline_files() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("demo");
        scaffold(&base("demo"), &target).unwrap();
        assert!(target.join("package.json").is_file());
        assert!(target.join("tsconfig.json").is_file());
        assert!(target.join("src/index.ts").is_file());
        assert!(target.join("src/index.test.ts").is_file());
        assert!(target.join(".nvmrc").is_file());
        let nvmrc = std::fs::read_to_string(target.join(".nvmrc")).unwrap();
        assert_eq!(nvmrc.trim(), "24");
    }

    #[test]
    fn ai_files_only_when_enabled() {
        let tmp = tempfile::tempdir().unwrap();
        let off = tmp.path().join("off");
        scaffold(&base("off"), &off).unwrap();
        assert!(!off.join("CLAUDE.md").exists());

        let mut c = base("on");
        c.ai = true;
        let on = tmp.path().join("on");
        scaffold(&c, &on).unwrap();
        assert!(on.join("CLAUDE.md").is_file());
        assert!(on.join("AGENTS.md").is_file());
    }

    #[test]
    fn refuses_non_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("demo");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("x.txt"), "hi").unwrap();
        assert!(scaffold(&base("demo"), &target).is_err());
    }

    #[test]
    fn esm_test_import_has_js_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("demo");
        scaffold(&base("demo"), &target).unwrap();
        let test = std::fs::read_to_string(target.join("src/index.test.ts")).unwrap();
        assert!(test.contains("./index.js"), "esm import should carry .js");
    }
}
