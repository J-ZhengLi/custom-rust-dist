
use std::path::{Path, PathBuf};

use crate::paths::TestPathExt;

struct FileBuilder {
    path: PathBuf,
}

impl FileBuilder {
    pub fn new(path: PathBuf) -> FileBuilder {
        FileBuilder {
            path
        }
    }

    fn mk(&mut self) {
        self.dirname().mkdir_p();
    }

    fn dirname(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

pub struct Project {
    root: PathBuf,
}

impl Project {
    pub fn new(root: PathBuf) -> Project {
        Project {
            root
        }
    }

    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }
}

pub struct ProjectBuilder {
    project: Project,
    files: Vec<FileBuilder>
}

impl ProjectBuilder {
    /// Generate test project
    pub fn from(root: PathBuf) -> ProjectBuilder {
        let root = Project::new(root);
        ProjectBuilder {
            project: root,
            files: vec![],
        }
    }

    pub fn file<B: AsRef<Path>>(mut self, path: B) -> Self {
        self.files.push(FileBuilder::new(
            self.project.root().join(path.as_ref())
        ));
        self
    }

    pub fn build(mut self) -> Project {
        // clean the home directory.
        self.project.root().rm_rf();
        // create the home directory
        self.project.root().mkdir_p();
        // create the extral file
        for file in self.files.iter_mut() {
            file.mk();
        }

        let ProjectBuilder { project, .. } = self;
        project
    }
}
