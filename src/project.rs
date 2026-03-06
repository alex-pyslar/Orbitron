use serde::Deserialize;
use std::{fs, path::Path};

// ── Manifest sections ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectSection,
    pub build:   BuildSection,
}

#[derive(Deserialize)]
pub struct ProjectSection {
    pub name:    String,
    pub version: String,
}

#[derive(Deserialize)]
pub struct BuildSection {
    /// Entry-point source file, relative to the project root.
    /// Default convention: "src/main.ot"
    pub main:   String,
    /// Output binary/jar path, relative to the project root.
    /// Default convention: "bin/<project-name>"
    pub output: String,
    /// Compilation backend: "llvm" (default) or "jvm"
    #[serde(default = "default_backend")]
    pub backend: String,
}

fn default_backend() -> String { "llvm".to_string() }

// ── Loader ───────────────────────────────────────────────────────────────────

/// Load and parse `orbitron.toml` from the given directory.
pub fn load_manifest(root: &Path) -> Result<ProjectManifest, String> {
    let toml_path = root.join("orbitron.toml");
    let text = fs::read_to_string(&toml_path)
        .map_err(|e| format!("Cannot read orbitron.toml: {e}"))?;
    toml::from_str(&text)
        .map_err(|e| format!("Error in orbitron.toml: {e}"))
}
