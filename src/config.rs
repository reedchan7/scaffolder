use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum PackageManager {
    Pnpm,
    Npm,
    Yarn,
    Bun,
}

impl PackageManager {
    pub fn bin(self) -> &'static str {
        match self {
            Self::Pnpm => "pnpm",
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Bun => "bun",
        }
    }

    /// Exact pinned `packageManager` field value. Refresh these on each release.
    pub fn package_manager_field(self) -> &'static str {
        match self {
            Self::Pnpm => "pnpm@11.5.2",
            Self::Npm => "npm@11.16.0",
            Self::Yarn => "yarn@1.22.22",
            Self::Bun => "bun@1.3.14",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum TestFramework {
    Vitest,
    Node,
    Bun,
}

impl TestFramework {
    pub fn default_for_pm(pm: PackageManager) -> Self {
        match pm {
            PackageManager::Bun => Self::Bun,
            PackageManager::Pnpm | PackageManager::Npm | PackageManager::Yarn => Self::Vitest,
        }
    }

    pub fn test_script(self) -> &'static str {
        match self {
            Self::Vitest => "vitest run",
            // Node's runner can't load `import` syntax from .ts source under
            // commonjs, so compile first and test the emitted JS (works for
            // both esm and cjs).
            Self::Node => "tsc && node --test \"dist/**/*.test.js\"",
            // Bun runs TS test files directly via its built-in runner.
            Self::Bun => "bun test",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum ModuleSystem {
    Esm,
    Cjs,
}

impl ModuleSystem {
    pub fn package_type(self) -> &'static str {
        match self {
            Self::Esm => "module",
            Self::Cjs => "commonjs",
        }
    }

    /// Extension required on relative imports under NodeNext resolution.
    pub fn import_ext(self) -> &'static str {
        match self {
            Self::Esm => ".js",
            Self::Cjs => "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum License {
    #[value(name = "MIT")]
    Mit,
    #[value(name = "Apache-2.0")]
    Apache2,
}

impl License {
    pub fn spdx(self) -> &'static str {
        match self {
            Self::Mit => "MIT",
            Self::Apache2 => "Apache-2.0",
        }
    }

    pub fn template_file(self) -> &'static str {
        match self {
            Self::Mit => "license-mit.txt",
            Self::Apache2 => "license-apache-2.0.txt",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScaffoldConfig {
    pub name: String,
    pub pm: PackageManager,
    pub test: TestFramework,
    pub module: ModuleSystem,
    pub node: u32,
    pub license: Option<License>,
    pub ai: bool,
    pub git: bool,
    pub install: bool,
}

impl ScaffoldConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        match (self.pm, self.test) {
            (PackageManager::Bun, TestFramework::Bun) => Ok(()),
            (PackageManager::Bun, _) => anyhow::bail!("--pm bun requires --test bun"),
            (_, TestFramework::Bun) => anyhow::bail!("--test bun requires --pm bun"),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_manager_strings() {
        assert_eq!(PackageManager::Pnpm.bin(), "pnpm");
        assert_eq!(PackageManager::Pnpm.package_manager_field(), "pnpm@11.5.2");
        assert_eq!(PackageManager::Npm.package_manager_field(), "npm@11.16.0");
        assert_eq!(PackageManager::Yarn.package_manager_field(), "yarn@1.22.22");
        assert_eq!(PackageManager::Bun.package_manager_field(), "bun@1.3.14");
    }

    #[test]
    fn module_system_mapping() {
        assert_eq!(ModuleSystem::Esm.package_type(), "module");
        assert_eq!(ModuleSystem::Cjs.package_type(), "commonjs");
        assert_eq!(ModuleSystem::Esm.import_ext(), ".js");
        assert_eq!(ModuleSystem::Cjs.import_ext(), "");
    }

    #[test]
    fn test_framework_script() {
        assert_eq!(
            TestFramework::default_for_pm(PackageManager::Pnpm),
            TestFramework::Vitest
        );
        assert_eq!(
            TestFramework::default_for_pm(PackageManager::Bun),
            TestFramework::Bun
        );
        assert_eq!(TestFramework::Vitest.test_script(), "vitest run");
        assert_eq!(
            TestFramework::Node.test_script(),
            "tsc && node --test \"dist/**/*.test.js\""
        );
        assert_eq!(TestFramework::Bun.test_script(), "bun test");
    }

    #[test]
    fn validates_bun_only_runtime_pairing() {
        let mut cfg = ScaffoldConfig {
            name: "demo".into(),
            pm: PackageManager::Bun,
            test: TestFramework::Bun,
            module: ModuleSystem::Esm,
            node: 24,
            license: None,
            ai: false,
            git: false,
            install: false,
        };
        assert!(cfg.validate().is_ok());

        cfg.test = TestFramework::Vitest;
        assert!(cfg.validate().is_err());

        cfg.pm = PackageManager::Pnpm;
        cfg.test = TestFramework::Bun;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn license_mapping() {
        assert_eq!(License::Mit.spdx(), "MIT");
        assert_eq!(License::Apache2.spdx(), "Apache-2.0");
        assert_eq!(License::Mit.template_file(), "license-mit.txt");
        assert_eq!(License::Apache2.template_file(), "license-apache-2.0.txt");
    }
}
