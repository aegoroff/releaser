use serde::{Deserialize};

extern crate semver;
extern crate toml;
extern crate serde;

#[derive(Deserialize)]
struct WorkspaceConfig {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    members: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_parse_workspace() {
        // Arrange
        let t = r#"
[workspace]

members = [
    "solv",
    "solp",
]
        "#;

        // Act
        let cfg: WorkspaceConfig = toml::from_str(t).unwrap();

        // Assert
        assert_eq!(2, cfg.workspace.members.len());
    }
}
