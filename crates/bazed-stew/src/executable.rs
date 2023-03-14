use std::path::{Path, PathBuf};

use semver::{Version, VersionReq};

#[derive(Debug, Clone)]
pub struct PluginExecutable(PathBuf);

impl std::fmt::Display for PluginExecutable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{} ({})",
            self.name(),
            self.version(),
            self.0.display()
        )
    }
}

impl PluginExecutable {
    pub fn new(path: PathBuf) -> Option<Self> {
        if !path.is_file() {
            return None;
        }
        let file_name = path.file_name()?.to_string_lossy();
        let version = file_name.split('@').nth(1)?.to_string();
        Version::parse(&version).is_ok().then_some(Self(path))
    }

    pub fn path(&self) -> &PathBuf {
        &self.0
    }

    pub fn name(&self) -> String {
        let file_name = self.0.file_name().unwrap().to_string_lossy();
        file_name.split('@').next().unwrap().to_string()
    }

    pub fn version(&self) -> Version {
        let file_name = self.0.file_name().unwrap().to_string_lossy();
        let version = file_name.split('@').nth(1).unwrap().to_string();
        Version::parse(&version).unwrap()
    }

    pub(crate) fn version_matches(&self, version_requirement: &VersionReq) -> bool {
        version_requirement.matches(&self.version())
    }
}

pub fn search_plugins_in(path: &Path) -> impl Iterator<Item = PluginExecutable> {
    path.read_dir()
        .unwrap()
        .filter_map(|entry| PluginExecutable::new(entry.unwrap().path()))
}

pub fn search_plugin(
    paths: &[PathBuf],
    name: &str,
    version_req: &VersionReq,
) -> Option<PluginExecutable> {
    paths
        .iter()
        .flat_map(|path| search_plugins_in(path))
        .find(|plugin| plugin.name() == name && plugin.version_matches(version_req))
}
