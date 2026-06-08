use clap::{Args, Parser, Subcommand};

use crate::config::{License, ModuleSystem, PackageManager, TestFramework};

#[derive(Parser, Debug)]
#[command(
    name = "scaffolder",
    version,
    about = "Multi-language project scaffolder"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a new project from a template.
    New(NewArgs),
    /// List available templates.
    List,
    /// Update scaffolder to the latest release.
    SelfUpdate,
}

#[derive(Args, Debug)]
pub struct NewArgs {
    /// Template id (default: typescript-node).
    pub template: Option<String>,
    /// Project name. When omitted, scaffolder runs interactively.
    pub name: Option<String>,

    #[arg(long, value_enum, default_value_t = PackageManager::Pnpm)]
    pub pm: PackageManager,
    #[arg(long, value_enum, default_value_t = TestFramework::Vitest)]
    pub test: TestFramework,
    #[arg(long, value_enum, default_value_t = ModuleSystem::Esm)]
    pub module: ModuleSystem,
    /// Node major version (integer only, e.g. 24).
    #[arg(long, default_value = "24")]
    pub node: String,
    /// Open-source license for the project. Omit this flag to keep the project
    /// private (sets "private": true in package.json, no LICENSE file).
    #[arg(long, value_enum)]
    pub license: Option<License>,
    /// Also generate CLAUDE.md + AGENTS.md AI guideline files.
    #[arg(long, default_value_t = false)]
    pub ai: bool,
    #[arg(long = "no-git", default_value_t = false)]
    pub no_git: bool,
    #[arg(long = "no-install", default_value_t = false)]
    pub no_install: bool,
}
