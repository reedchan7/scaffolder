use owo_colors::OwoColorize;

use crate::cli::NewArgs;
use crate::config::ScaffoldConfig;
use crate::interactive::prompt_config;
use crate::render::{normalize_package_name, parse_node};
use crate::{postops, scaffold};

const DEFAULT_TEMPLATE: &str = "typescript-node";

pub fn run(args: NewArgs) -> anyhow::Result<()> {
    let (template, name) = resolve_template_and_name(args.template.clone(), args.name.clone());
    if template != DEFAULT_TEMPLATE {
        anyhow::bail!("unknown template {template:?}; available: {DEFAULT_TEMPLATE}");
    }

    let cfg = build_config(args, name)?;

    let target = std::env::current_dir()?.join(&cfg.name);
    scaffold::scaffold(&cfg, &target)?;

    if cfg.git {
        postops::git_init(&target)?;
    }
    if cfg.install {
        postops::install(&target, cfg.pm)?;
    }

    print_next_steps(&cfg);
    Ok(())
}

/// Disambiguate the two optional positionals `[TEMPLATE] [NAME]`.
///
/// With a single template, `scaffolder new my-app` should treat `my-app` as the
/// project name, not a (missing) template. Rules:
/// - both given            → (template, Some(name))
/// - only first, == known  → (template, None)  → interactive name
/// - only first, != known  → (default, Some(first))  → first is the name
/// - neither               → (default, None)  → fully interactive
fn resolve_template_and_name(
    first: Option<String>,
    second: Option<String>,
) -> (String, Option<String>) {
    match (first, second) {
        (Some(t), Some(n)) => (t, Some(n)),
        (Some(first), None) => {
            if first == DEFAULT_TEMPLATE {
                (first, None)
            } else {
                (DEFAULT_TEMPLATE.to_string(), Some(first))
            }
        }
        (None, _) => (DEFAULT_TEMPLATE.to_string(), None),
    }
}

/// If a name is resolved, use flags + defaults (no prompts → CI friendly).
/// Otherwise prompt interactively, seeding prompts from the supplied flags.
fn build_config(args: NewArgs, name: Option<String>) -> anyhow::Result<ScaffoldConfig> {
    match name {
        None => prompt_config(&args),
        Some(raw) => Ok(ScaffoldConfig {
            name: normalize_package_name(&raw)?,
            pm: args.pm,
            test: args.test,
            module: args.module,
            node: parse_node(&args.node)?,
            license: args.license,
            ai: args.ai,
            git: !args.no_git,
            install: !args.no_install,
        }),
    }
}

fn print_next_steps(cfg: &ScaffoldConfig) {
    println!("\n{} {}\n", "Created".green().bold(), cfg.name.bold());
    println!("Next steps:");
    println!("  cd {}", cfg.name);
    if !cfg.install {
        println!("  {} install", cfg.pm.bin());
    }
    println!("  make check");
}
