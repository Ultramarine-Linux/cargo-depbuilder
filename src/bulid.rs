use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct AndaConfig {
    pub project: HashMap<String, Project>,
}

#[derive(Serialize, PartialEq, Eq)]
pub struct Project {
    pub rpmbuild: RpmBuild,
}

#[derive(Serialize, PartialEq, Eq)]
pub struct RpmBuild {
    pub spec: PathBuf,
    // pub mode: String,
    pub package: String,
    pub build_deps: Vec<String>,
}

impl AndaConfig {
    pub fn new() -> Self {
        Self {
            project: HashMap::new(),
        }
    }
    pub fn add(&mut self, name: &String, specfile: String, build_deps: Vec<String>) {
        self.project.insert(
            String::from(name),
            Project {
                rpmbuild: RpmBuild {
                    spec: PathBuf::from(format!("{}/{}", name, specfile)),
                    // mode: String::from("rpmbuild"),
                    package: format!("rust-{}", name).to_string(),
                    build_deps,
                },
            },
        );
    }
    pub fn hcl(&self) -> Result<()> {
        let path = std::path::Path::new("anda.hcl");
        Ok(hcl::to_writer(
            (if path.exists() {
                File::options().write(true).open(path)
            } else {
                File::create(path)
            })?,
            self,
        )?)
    }
}
