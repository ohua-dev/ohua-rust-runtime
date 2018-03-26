use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum CodeGenerationError {
    TargetDirNotCreated(io::Error),
    InconsistentMainArity,
    ModuleDirUndeletable(io::Error),
    ModuleDirNotCreated(io::Error),
    StaticPopulationFailed(io::Error),
    CastGenerationFailed(io::Error),
    WrapperGenerationFailed(io::Error),
    ModfileGenerationFailed(io::Error),
    RuntimeDataCreationFailed(io::Error),
}

impl fmt::Display for CodeGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CodeGenerationError::*;
        match *self {
            TargetDirNotCreated(ref io_err) => format!("Unable to create the target directory. {}", io_err.description()).fmt(f),
            InconsistentMainArity => "The number of arguments specified in the OhuaData structure and the `type_dump` file don't match.".fmt(f),
            ModuleDirUndeletable(ref io_err) => format!("Unable to remove the previously generated runtime folder. {}", io_err).fmt(f),
            ModuleDirNotCreated(ref io_err) => format!("Unable to create the module directory for the ohua runtime. {}", io_err).fmt(f),
            StaticPopulationFailed(ref io_err) => format!("The static `ohua_runtime` module folder population failed unexpectedly. {}", io_err).fmt(f),
            CastGenerationFailed(ref io_err) => format!("Unable to create the generic type file. {}", io_err).fmt(f),
            WrapperGenerationFailed(ref io_err) => format!("Unable to create the function wrappers. {}", io_err).fmt(f),
            ModfileGenerationFailed(ref io_err) => format!("Unable to create the module file. {}", io_err).fmt(f),
            RuntimeDataCreationFailed(ref io_err) => format!("Unable to create the runtime data structure file. {}", io_err).fmt(f),
        }
    }
}

impl Error for CodeGenerationError {
    fn description(&self) -> &str {
        use CodeGenerationError::*;
        match *self {
            TargetDirNotCreated(_) => "Target dir creation failed",
            InconsistentMainArity => "Inconsistent MainArity",
            ModuleDirUndeletable(_) => "Deletion of old runtime failed",
            ModuleDirNotCreated(_) => "Could not create module dir",
            StaticPopulationFailed(_) => "Static population failed",
            CastGenerationFailed(_) => "GenericType file creation failed",
            WrapperGenerationFailed(_) => "Wrapper generation failed",
            ModfileGenerationFailed(_) => "ModFile generation failed",
            RuntimeDataCreationFailed(_) => "Runtime Structure generation failed",
        }
    }
}
