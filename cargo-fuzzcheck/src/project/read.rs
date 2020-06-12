

use crate::project::*;

use std::result::Result;
use std::path::{Path, PathBuf};

use std::fs;

use std::io;
use std::io::Read;

use std::collections::HashMap;


impl Root {
    pub fn from_path(root_folder: &Path) -> Result<Self, RootError> {
        
        let cargo_toml_path = root_folder.join("Cargo.toml");
        let cargo_toml_file = fs::File::open(&cargo_toml_path).map_err(|e| RootError::CannotReadCargoToml(cargo_toml_path.clone(), e))?;
        let cargo_toml = CargoToml::from_file(cargo_toml_file)?;

        let name = cargo_toml.toml
            .as_table()
            .and_then(|v| v.get("package"))
            .and_then(|v| v.get("name"))
            .ok_or(RootError::CannotFindNameInCargoToml)?;

        let fuzz_path = root_folder.join("fuzz");
        let _ = fs::read_dir(&fuzz_path).map_err(|e| RootError::CannotReadFuzz(fuzz_path.clone(), e))?;
        let fuzz = Fuzz::from_path(&fuzz_path)?;

        Ok(Self {
            name: name.to_string(),
            fuzz,
            cargo_toml,
        })
    }
}

impl Fuzz {
    pub fn from_path(fuzz_folder: &Path) -> Result<Self, FuzzError> {
        let non_instr_path = fuzz_folder.join("non_instrumented");
        let _ = fs::read_dir(non_instr_path.clone()).map_err(|e| FuzzError::CannotReadNonInstrumented(non_instr_path.clone(), e))?;
        let non_instrumented = NonInstrumented::from_path(&non_instr_path)?;

        let instr_path = fuzz_folder.join("instrumented");
        let _ = fs::read_dir(instr_path.clone()).map_err(|e| FuzzError::CannotReadInstrumented(instr_path.clone(), e))?;
        let instrumented = Instrumented::from_path(&instr_path)?;

        let corpora_path = fuzz_folder.join("corpora");
        let corpora: Result<Corpora, CorporaError> = {
            match fs::read_dir(corpora_path.clone()) {
                Ok(_) => Corpora::from_path(&corpora_path),
                Err(e) => Err(e.into())
            }
        };

        let gitignore_path = fuzz_folder.join(".gitignore");
        let gitignore = 
            fs::File::open(gitignore_path)
            .and_then(|mut f| {
                let mut string = String::new();
                let _ = f.read_to_string(&mut string)?;
                Ok(string)
            }).ok();

        Ok(Self {
            non_instrumented,
            instrumented,
            corpora,
            gitignore,
        })
    }
}

impl NonInstrumented {
    pub fn from_path(non_instrumented_folder: &Path) -> Result<Self, NonInstrumentedError> {
        let fuzz_targets_path = non_instrumented_folder.join("fuzz_targets");
        let _ = fs::read_dir(fuzz_targets_path.clone()).map_err(|e| NonInstrumentedError::CannotReadFuzzTargets(fuzz_targets_path.clone(), e))?;
        let fuzz_targets = FuzzTargets::from_path(&fuzz_targets_path)?;

        let build_rs_path = non_instrumented_folder.join("build.rs");
        let build_rs_file = fs::File::open(&build_rs_path).map_err(|e| NonInstrumentedError::CannotReadBuildRs(build_rs_path.clone(), e))?;
        let build_rs = BuildRs::from_file(build_rs_file)?;

        let cargo_toml_path = non_instrumented_folder.join("Cargo.toml");
        let cargo_toml_file = fs::File::open(&cargo_toml_path).map_err(|e| NonInstrumentedError::CannotReadCargoToml(cargo_toml_path.clone(), e))?;
        let cargo_toml = CargoToml::from_file(cargo_toml_file)?;

        Ok(Self {
            fuzz_targets,
            build_rs,
            cargo_toml,
        })
    }
}

impl FuzzTargets {
    pub fn from_path(non_instrumented_folder: &Path) -> Result<Self, FuzzTargetsError> {
        let folder = fs::read_dir(non_instrumented_folder).map_err(|e| FuzzTargetsError::IoError(e))?;

        let mut targets = HashMap::new();

        let mut errors = Vec::<FuzzTargetError>::new();

        for result in folder {
            match result {
                Ok(entry) => {
                    let path = entry.path();
                    if !path.is_file() {
                        continue
                    }
                    if path.extension().and_then(|ex| ex.to_str()) != Some("rs") { 
                        errors.push(FuzzTargetError::ExtensionError(path.to_path_buf()));
                        continue 
                    }
                    let mut content = Vec::new();
                    match fs::File::open(entry.path()) {
                        Ok(mut file) => match file.read_to_end(&mut content) {
                            Ok(_) => { targets.insert(entry.file_name(), content); },
                            Err(e) => { errors.push(e.into()); },
                        },
                        Err(e) => errors.push(e.into()),
                    }
                }
                Err(e) => {
                    errors.push(e.into())
                }
            }
        }

        if targets.is_empty() {
            Err(FuzzTargetsError::NoFuzzTargets(errors))
        } else {
            Ok(Self {
                targets,
            })
        }
    }
}

impl BuildRs {
    pub fn from_file(mut file: fs::File) -> Result<BuildRs, BuildRsError> {
        let mut content = Vec::new();
        let _ = file.read_to_end(&mut content)?;

        Ok(BuildRs {
            content,
        })
    }
}

impl Instrumented {
    pub fn from_path(instrumented_folder: &Path) -> Result<Self, InstrumentedError> {
        let cargo_toml_path = instrumented_folder.join("Cargo.toml");
        let cargo_toml_file = fs::File::open(&cargo_toml_path).map_err(|e| InstrumentedError::CannotReadCargoToml(cargo_toml_path.clone(), e))?;
        let cargo_toml = CargoToml::from_file(cargo_toml_file)?;

        Ok(Self {
            cargo_toml,
        })
    }
}

impl Corpora {
    pub fn from_path(corpora_folder: &Path) -> Result<Self, CorporaError> {
        let folder = fs::read_dir(corpora_folder).map_err(|e| CorporaError::IoError(e))?;

        let mut corpora = Vec::new();

        let mut errors = Vec::<CorporaError>::new();
        
        for result in folder {
            match result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() { 
                        errors.push(CorporaError::CorpusIsNotDirectory(path.clone()));
                        continue 
                    }
                    if let Err(e) = fs::read_dir(&path) {
                        errors.push(e.into());
                    } else {
                        corpora.push(path);
                    }
                }
                Err(e) => {
                    errors.push(e.into())
                }
            }
        }

        Ok(Self {
            corpora,
        })
    }
}

impl CargoToml {
    pub fn from_file(mut file: fs::File) -> Result<CargoToml, CargoTomlError> {
        let mut content = Vec::new();
        let _ = file.read_to_end(&mut content)?;
        let toml: toml::Value = toml::from_slice(&content)?;

        Ok(CargoToml {
            toml,
        })
    }
}

#[derive(Debug)]
pub enum RootError {
    IoError(io::Error),
    CannotReadCargoToml(PathBuf, io::Error),
    CargoToml(CargoTomlError),
    CannotFindNameInCargoToml,
    CannotReadFuzz(PathBuf, io::Error),
    FuzzError(FuzzError),
}
impl From<FuzzError> for RootError {
    fn from(e: FuzzError) -> Self {
        Self::FuzzError(e)
    }
}
impl From<io::Error> for RootError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}
impl From<CargoTomlError> for RootError {
    fn from(e: CargoTomlError) -> Self {
        Self::CargoToml(e)
    }
}

#[derive(Debug)]
pub enum FuzzError {
    CannotReadNonInstrumented(PathBuf, io::Error),
    CannotReadInstrumented(PathBuf, io::Error),
    NonInstrumented(NonInstrumentedError),
    Instrumented(InstrumentedError),
}
impl From<NonInstrumentedError> for FuzzError {
    fn from(e: NonInstrumentedError) -> Self {
        Self::NonInstrumented(e)
    }
}
impl From<InstrumentedError> for FuzzError {
    fn from(e: InstrumentedError) -> Self {
        Self::Instrumented(e)
    }
}

#[derive(Debug)]
pub enum NonInstrumentedError {
    CannotReadFuzzTargets(PathBuf, io::Error),
    FuzzTargets(FuzzTargetsError),
    CannotReadBuildRs(PathBuf, io::Error),
    BuildRs(BuildRsError),
    CannotReadCargoToml(PathBuf, io::Error),
    CargoToml(CargoTomlError),
}
impl From<FuzzTargetsError> for NonInstrumentedError {
    fn from(e: FuzzTargetsError) -> Self {
        Self::FuzzTargets(e)
    }
}
impl From<BuildRsError> for NonInstrumentedError {
    fn from(e: BuildRsError) -> Self {
        Self::BuildRs(e)
    }
}
impl From<CargoTomlError> for NonInstrumentedError {
    fn from(e: CargoTomlError) -> Self {
        Self::CargoToml(e)
    }
}

#[derive(Debug)]
pub enum FuzzTargetsError {
    IoError(io::Error),
    NoFuzzTargets(Vec<FuzzTargetError>),
}
impl From<io::Error> for FuzzTargetsError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

#[derive(Debug)]
pub enum FuzzTargetError {
    IoError(io::Error),
    ExtensionError(PathBuf),
}
impl From<io::Error> for FuzzTargetError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

#[derive(Debug)]
pub enum BuildRsError {
    IoError(io::Error)
}
impl From<io::Error> for BuildRsError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

#[derive(Debug)]
pub enum InstrumentedError {
    CannotReadCargoToml(PathBuf, io::Error),
    CargoToml(CargoTomlError),
}
impl From<CargoTomlError> for InstrumentedError {
    fn from(e: CargoTomlError) -> Self {
        Self::CargoToml(e)
    }
}

#[derive(Debug)]
pub enum CorporaError {
    IoError(io::Error),
    CorpusIsNotDirectory(PathBuf),
}
impl From<io::Error> for CorporaError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

#[derive(Debug)]
pub enum CargoTomlError {
    IoError(io::Error),
    CannotParseToml(toml::de::Error),
}
impl From<io::Error> for CargoTomlError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}
impl From<toml::de::Error> for CargoTomlError {
    fn from(e: toml::de::Error) -> Self {
        Self::CannotParseToml(e)
    }
}