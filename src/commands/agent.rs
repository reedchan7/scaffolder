use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use inquire::Confirm;
use serde_json::{Map, Value};
use toml_edit::{DocumentMut, Item, Table, value};

use crate::cli::{AgentArgs, AgentCommand, AgentKind};

pub fn run(args: AgentArgs) -> anyhow::Result<()> {
    match args.command {
        AgentCommand::Trust(args) => {
            for agent in args.agents {
                trust(agent)?;
            }
            Ok(())
        }
    }
}

fn trust(agent: AgentKind) -> anyhow::Result<()> {
    let project = env::current_dir()?.canonicalize()?;
    let home = home_dir()?;

    match agent {
        AgentKind::Claude => trust_claude(&project)?,
        AgentKind::Codex => trust_codex(&home, &project)?,
        AgentKind::KimiCode => trust_kimi_code(&home)?,
        AgentKind::Agy => trust_agy(&home, &project)?,
        AgentKind::Reasonix => trust_reasonix(&project)?,
        AgentKind::Pi => trust_pi(&home, &project)?,
    }

    println!("Trusted {} for {}", agent.as_str(), project.display());
    Ok(())
}

fn trust_claude(project: &Path) -> anyhow::Result<()> {
    set_json_string(
        &project.join(".claude/settings.local.json"),
        &["permissions", "defaultMode"],
        "bypassPermissions",
    )
}

fn trust_codex(home: &Path, project: &Path) -> anyhow::Result<()> {
    let project_config = project.join(".codex/config.toml");
    set_toml_string(&project_config, &["approval_policy"], "never")?;
    set_toml_string(&project_config, &["sandbox_mode"], "danger-full-access")?;

    set_toml_string(
        &home.join(".codex/config.toml"),
        &[
            "projects",
            project.to_string_lossy().as_ref(),
            "trust_level",
        ],
        "trusted",
    )
}

fn trust_kimi_code(home: &Path) -> anyhow::Result<()> {
    set_toml_string(
        &home.join(".kimi-code/config.toml"),
        &["default_permission_mode"],
        "yolo",
    )
}

fn trust_agy(home: &Path, project: &Path) -> anyhow::Result<()> {
    let settings = home.join(".gemini/antigravity-cli/settings.json");
    set_json_string(&settings, &["toolPermission"], "always-proceed")?;
    set_json_array_contains_string(
        &settings,
        &["trustedWorkspaces"],
        &project.to_string_lossy(),
    )
}

fn trust_reasonix(project: &Path) -> anyhow::Result<()> {
    set_toml_string(
        &project.join("reasonix.toml"),
        &["permissions", "mode"],
        "allow",
    )
}

fn trust_pi(home: &Path, project: &Path) -> anyhow::Result<()> {
    let path = home.join(".pi/agent/trust.json");
    let mut root = read_json_object(&path)?;
    let key = project.to_string_lossy().into_owned();

    match root.get(&key) {
        Some(Value::Bool(true)) => {}
        Some(existing) => {
            confirm_replace(&path, &key, &json_value_label(existing), "true")?;
            root.insert(key, Value::Bool(true));
        }
        None => {
            root.insert(key, Value::Bool(true));
        }
    }

    write_json_object(&path, root)
}

fn home_dir() -> anyhow::Result<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .context("could not determine home directory")
}

fn set_json_string(path: &Path, key_path: &[&str], target: &str) -> anyhow::Result<()> {
    let mut root = read_json_object(path)?;
    set_json_string_value(&mut root, path, key_path, target)?;
    write_json_object(path, root)
}

fn set_json_string_value(
    root: &mut Map<String, Value>,
    path: &Path,
    key_path: &[&str],
    target: &str,
) -> anyhow::Result<()> {
    let (last, parents) = key_path
        .split_last()
        .context("JSON key path must not be empty")?;
    let mut current = root;

    for segment in parents {
        current = json_child_object(current, path, segment)?;
    }

    match current.get(*last) {
        Some(Value::String(existing)) if existing == target => {}
        Some(existing) => {
            confirm_replace(
                path,
                &key_path.join("."),
                &json_value_label(existing),
                target,
            )?;
            current.insert((*last).to_string(), Value::String(target.to_string()));
        }
        None => {
            current.insert((*last).to_string(), Value::String(target.to_string()));
        }
    }

    Ok(())
}

fn json_child_object<'a>(
    object: &'a mut Map<String, Value>,
    path: &Path,
    key: &str,
) -> anyhow::Result<&'a mut Map<String, Value>> {
    let should_replace = match object.get(key) {
        Some(Value::Object(_)) => false,
        Some(existing) => {
            confirm_replace(path, key, &json_value_label(existing), "object")?;
            true
        }
        None => true,
    };

    if should_replace {
        object.insert(key.to_string(), Value::Object(Map::new()));
    }

    object
        .get_mut(key)
        .and_then(Value::as_object_mut)
        .context("JSON object was not created")
}

fn set_json_array_contains_string(
    path: &Path,
    key_path: &[&str],
    target: &str,
) -> anyhow::Result<()> {
    let mut root = read_json_object(path)?;
    let (last, parents) = key_path
        .split_last()
        .context("JSON key path must not be empty")?;
    let mut current = &mut root;

    for segment in parents {
        current = json_child_object(current, path, segment)?;
    }

    match current.get_mut(*last) {
        Some(Value::Array(items)) => {
            if !items.iter().any(|item| item.as_str() == Some(target)) {
                items.push(Value::String(target.to_string()));
            }
        }
        Some(existing) => {
            confirm_replace(
                path,
                &key_path.join("."),
                &json_value_label(existing),
                "array",
            )?;
            current.insert(
                (*last).to_string(),
                Value::Array(vec![Value::String(target.to_string())]),
            );
        }
        None => {
            current.insert(
                (*last).to_string(),
                Value::Array(vec![Value::String(target.to_string())]),
            );
        }
    }

    write_json_object(path, root)
}

fn read_json_object(path: &Path) -> anyhow::Result<Map<String, Value>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Map::new()),
        Err(err) => return Err(err).with_context(|| format!("failed to read {}", path.display())),
    };

    if contents.trim().is_empty() {
        return Ok(Map::new());
    }

    match serde_json::from_str::<Value>(&contents)
        .with_context(|| format!("failed to parse {}", path.display()))?
    {
        Value::Object(object) => Ok(object),
        other => {
            confirm_replace(path, "<root>", &json_value_label(&other), "object")?;
            Ok(Map::new())
        }
    }
}

fn write_json_object(path: &Path, object: Map<String, Value>) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let rendered = serde_json::to_string_pretty(&Value::Object(object))? + "\n";
    fs::write(path, rendered).with_context(|| format!("failed to write {}", path.display()))
}

fn json_value_label(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => format!("{value:?}"),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

fn set_toml_string(path: &Path, key_path: &[&str], target: &str) -> anyhow::Result<()> {
    let mut doc = read_toml_document(path)?;
    set_toml_string_value(&mut doc, path, key_path, target)?;
    write_toml_document(path, &doc)
}

fn set_toml_string_value(
    doc: &mut DocumentMut,
    path: &Path,
    key_path: &[&str],
    target: &str,
) -> anyhow::Result<()> {
    let (last, parents) = key_path
        .split_last()
        .context("TOML key path must not be empty")?;
    let mut current = doc.as_table_mut();

    for segment in parents {
        current = toml_child_table(current, path, segment)?;
    }

    match current.get(last) {
        Some(item) if toml_item_str(item) == Some(target) => {}
        Some(item) => {
            confirm_replace(path, &key_path.join("."), &toml_item_label(item), target)?;
            current.insert(last, value(target));
        }
        None => {
            current.insert(last, value(target));
        }
    }

    Ok(())
}

fn toml_child_table<'a>(
    table: &'a mut Table,
    path: &Path,
    key: &str,
) -> anyhow::Result<&'a mut Table> {
    let should_replace = match table.get(key) {
        Some(Item::Table(_)) => false,
        Some(item) => {
            confirm_replace(path, key, &toml_item_label(item), "table")?;
            true
        }
        None => true,
    };

    if should_replace {
        table.insert(key, Item::Table(Table::new()));
    }

    table
        .get_mut(key)
        .and_then(Item::as_table_mut)
        .context("TOML table was not created")
}

fn read_toml_document(path: &Path) -> anyhow::Result<DocumentMut> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err).with_context(|| format!("failed to read {}", path.display())),
    };

    contents
        .parse::<DocumentMut>()
        .with_context(|| format!("failed to parse {}", path.display()))
}

fn write_toml_document(path: &Path, doc: &DocumentMut) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    fs::write(path, doc.to_string()).with_context(|| format!("failed to write {}", path.display()))
}

fn toml_item_str(item: &Item) -> Option<&str> {
    item.as_value().and_then(|value| value.as_str())
}

fn toml_item_label(item: &Item) -> String {
    let rendered = item.to_string();
    let trimmed = rendered.trim();
    if trimmed.is_empty() {
        "empty".to_string()
    } else {
        trimmed.to_string()
    }
}

fn confirm_replace(path: &Path, key: &str, current: &str, target: &str) -> anyhow::Result<()> {
    let prompt = format!(
        "{} has {key} set to {current}. Replace with {target}?",
        path.display()
    );

    if Confirm::new(&prompt).with_default(false).prompt()? {
        Ok(())
    } else {
        bail!("left {key} unchanged in {}", path.display())
    }
}
