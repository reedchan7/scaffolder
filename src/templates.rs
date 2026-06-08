use rust_embed::RustEmbed;

/// All static template files for the TypeScript + Node.js scaffold.
#[derive(RustEmbed)]
#[folder = "templates/typescript-node"]
pub struct TsNodeTemplate;

/// AI collaboration guideline snapshots, pulled verbatim from the repo root
/// at compile time so the embedded copy never drifts from the source of truth.
pub const CLAUDE_MD: &str = include_str!("../CLAUDE.md");
pub const AGENTS_MD: &str = include_str!("../AGENTS.md");

/// Fetch an embedded template file as UTF-8 text.
pub fn get(path: &str) -> anyhow::Result<String> {
    let file = TsNodeTemplate::get(path)
        .ok_or_else(|| anyhow::anyhow!("missing embedded template: {path}"))?;
    Ok(String::from_utf8(file.data.into_owned())?)
}
