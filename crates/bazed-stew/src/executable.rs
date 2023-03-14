use std::path::{Path, PathBuf};

use semver::{Version, VersionReq};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    VersionParse(#[from] semver::Error),
    #[error("Malformed plugin filename, must be \"<name>@<version>\"")]
    MalformedName(String),
    #[error("Given path is not a file")]
    NotAFile(PathBuf),
}

#[derive(Debug, Clone)]
pub struct PluginExecutable {
    pub name: String,
    pub version: Version,
    pub path: PathBuf,
}

impl std::fmt::Display for PluginExecutable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{} ({})",
            self.name,
            self.version,
            self.path.display()
        )
    }
}

impl PluginExecutable {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let file_name = &path
            .file_name()
            .ok_or_else(|| Error::NotAFile(path.clone()))?
            .to_str()
            .expect("Non-unicode filename encountered")
            .to_string();
        let (name, version) = file_name
            .split_once('@')
            .ok_or_else(|| Error::MalformedName(file_name.clone()))?;
        Ok(Self {
            name: name.to_string(),
            version: Version::parse(&version)?,
            path,
        })
    }

    pub fn filename(&self) -> &str {
        self.path
            .file_name()
            .unwrap()
            .to_str()
            .expect("Non-unicode filename encountered")
    }

    pub(crate) fn version_matches(&self, version_requirement: &VersionReq) -> bool {
        version_requirement.matches(&self.version)
    }

}

pub fn search_plugins_in(path: &Path) -> impl Iterator<Item = PluginExecutable> {
    path.read_dir()
        .unwrap()
        .filter_map(|entry| PluginExecutable::new(entry.unwrap().path()).ok())
}

pub fn search_plugin(
    paths: &[PathBuf],
    name: &str,
    version_req: &VersionReq,
) -> Option<PluginExecutable> {
    paths
        .iter()
        .flat_map(|path| search_plugins_in(path))
        .find(|plugin| plugin.name == name && plugin.version_matches(version_req))
}
