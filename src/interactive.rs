use inquire::{Confirm, Select, Text};

use crate::cli::NewArgs;
use crate::config::{License, ModuleSystem, PackageManager, ScaffoldConfig, TestFramework};
use crate::render::{normalize_package_name, parse_node};

/// Collect a full config interactively. Used when `new` is run without a NAME.
/// Supplied flags seed each prompt's default so they are respected, not ignored.
pub fn prompt_config(args: &NewArgs) -> anyhow::Result<ScaffoldConfig> {
    let raw_name = Text::new("Project name:").prompt()?;
    let name = normalize_package_name(&raw_name)?;

    let pm = select_enum(
        "Package manager:",
        &[
            ("pnpm", PackageManager::Pnpm),
            ("npm", PackageManager::Npm),
            ("yarn", PackageManager::Yarn),
            ("bun", PackageManager::Bun),
        ],
        args.pm,
    )?;
    let test = if pm == PackageManager::Bun || args.test == Some(TestFramework::Bun) {
        args.test
            .unwrap_or_else(|| TestFramework::default_for_pm(pm))
    } else {
        select_enum(
            "Test framework:",
            &[
                ("vitest", TestFramework::Vitest),
                ("node", TestFramework::Node),
            ],
            args.test
                .unwrap_or_else(|| TestFramework::default_for_pm(pm)),
        )?
    };
    let module = select_enum(
        "Module system:",
        &[("esm", ModuleSystem::Esm), ("cjs", ModuleSystem::Cjs)],
        args.module,
    )?;

    let node_raw = Text::new("Node major version:")
        .with_default(&args.node)
        .prompt()?;
    let node = parse_node(&node_raw)?;

    let license = select_enum(
        "License:",
        &[
            ("none (private)", None),
            ("MIT", Some(License::Mit)),
            ("Apache-2.0", Some(License::Apache2)),
        ],
        args.license,
    )?;

    let ai = Confirm::new("Generate AI guideline files (CLAUDE.md/AGENTS.md)?")
        .with_default(args.ai)
        .prompt()?;
    let git = Confirm::new("Initialize a git repository?")
        .with_default(!args.no_git)
        .prompt()?;
    let install = Confirm::new("Install dependencies now?")
        .with_default(!args.no_install)
        .prompt()?;

    Ok(ScaffoldConfig {
        name,
        pm,
        test,
        module,
        node,
        license,
        ai,
        git,
        install,
    })
}

/// Render a Select whose cursor starts on the value matching `seed`.
fn select_enum<T: Copy + PartialEq>(
    prompt: &str,
    options: &[(&str, T)],
    seed: T,
) -> anyhow::Result<T> {
    let labels: Vec<&str> = options.iter().map(|(l, _)| *l).collect();
    let start = options.iter().position(|(_, v)| *v == seed).unwrap_or(0);
    let chosen = Select::new(prompt, labels)
        .with_starting_cursor(start)
        .prompt()?;
    let value = options
        .iter()
        .find(|(l, _)| *l == chosen)
        .map(|(_, v)| *v)
        .expect("selection always matches an option");
    Ok(value)
}
