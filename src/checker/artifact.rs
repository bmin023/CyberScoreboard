use std::fs;
use std::io::Write;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::current_time;

/// Represents an uploaded file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Artifact {
    uuid: Uuid,
    original_name: String,
    name: String,
    path: String,
    upload_time: u128,
    deleted: bool,
}

impl Artifact {
    pub fn full_path(&self) -> String {
        format!("{}/{}", self.path, self.name)
    }
}

pub struct ArtifactBuilder {
    artifact: Artifact,
    correct_extension: bool,
    overwrite: bool,
}

impl ArtifactBuilder {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        Self {
            artifact: Artifact {
                uuid,
                original_name: uuid.to_string(),
                name: uuid.to_string(),
                path: "".to_string(),
                upload_time: current_time(),
                deleted: false,
            },
            correct_extension: false,
            overwrite: false,
        }
    }
    /// The path to the directory where this file will be stores
    /// (will attempt to create the path if it does not exist.
    pub fn path(mut self, path: String) -> Self {
        self.artifact.path = path;
        self
    }

    pub fn original_name(mut self, original_name: String) -> Self {
        self.artifact.original_name = original_name;
        self
    }

    /// What will the filename show up as in the directory
    /// If the name already exists at the location of choosing,
    /// will append a number to it to make the filename unique
    /// if overwrite is not specified.
    pub fn name(mut self, filename: String) -> Self {
        self.artifact.name = filename;
        self
    }

    pub fn overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    /// Sets the correct extension based off of the original filename.
    /// If the filename is "foo" and the original filename is "bar.tar.gz",
    /// The new filename will be "foo.tar.gz".
    pub fn with_correct_ext(mut self) -> Self {
        self.correct_extension = true;
        self
    }

    /// Builds the Artifact and creates the file with
    /// the data provided.
    pub fn build(mut self, data: &[u8]) -> Result<Artifact, ArtifactError> {
        fs::create_dir_all(&self.artifact.path).map_err(|_| ArtifactError::FileError)?;
        let mut created = false;
        let mut addition: Option<u8> = None;
        while !created {
            let true_name = {
                let addition_str = match addition {
                    None => "".to_string(),
                    Some(a) => a.to_string(),
                };
                let mut name = self.artifact.name + &addition_str;
                if self.correct_extension {
                    if let Some(i) = self.artifact.original_name.split_once('.') {
                        name = name + i.1;
                    }
                }
                name
            };
            self.artifact.name = true_name;
            let full_path = format!("{}/{}", &self.artifact.path, &self.artifact.name);
            let creation = if self.overwrite {
                fs::File::create(full_path)
            } else {
                fs::File::create_new(full_path)
            };
            if let Ok(mut file) = creation {
                file.write_all(data).map_err(|_| ArtifactError::FileError)?;
                created = true;
            }
            addition = match addition {
                Some(a) => Some(a + 1),
                None => Some(1),
            };
            if let Some(255) = addition {
                return Err(ArtifactError::NoSpace);
            }
        }
        Ok(self.artifact)
    }
}

pub enum ArtifactError {
    FileError,
    NoSpace,
}
