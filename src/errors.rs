use std::error::Error;
use std::fmt;
use std::io;
use syn::synom::ParseError;

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
    OperatorCreationFailed(io::Error),
}

impl fmt::Display for CodeGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CodeGenerationError::*;
        match *self {
            TargetDirNotCreated(ref io_err) => write!(f, "Unable to create the target directory. {}", io_err.description()),
            InconsistentMainArity => "The number of arguments specified in the OhuaData structure and the `type_dump` file don't match.".fmt(f),
            ModuleDirUndeletable(ref io_err) => write!(f, "Unable to remove the previously generated runtime folder. {}", io_err),
            ModuleDirNotCreated(ref io_err) => write!(f, "Unable to create the module directory for the ohua runtime. {}", io_err),
            StaticPopulationFailed(ref io_err) => write!(f, "The static `ohua_runtime` module folder population failed unexpectedly. {}", io_err),
            CastGenerationFailed(ref io_err) => write!(f, "Unable to create the generic type file. {}", io_err),
            WrapperGenerationFailed(ref io_err) => write!(f, "Unable to create the function wrappers. {}", io_err),
            ModfileGenerationFailed(ref io_err) => write!(f, "Unable to create the module file. {}", io_err),
            RuntimeDataCreationFailed(ref io_err) => write!(f, "Unable to create the runtime data structure file. {}", io_err),
            OperatorCreationFailed(ref io_err) => write!(f, "Unable to create the operator modules. {}", io_err),
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
            OperatorCreationFailed(_) => "Operator Module generation failed"
        }
    }
}

#[derive(Debug)]
pub enum TypeExtractionError {
    IOError(io::Error),
    ModuleNotFound(String),
    FunctionNotFoundInModule(String, String),
    ParsingError(ParseError),
    MalformedAlgoImportSpec(String),
}

impl fmt::Display for TypeExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TypeExtractionError::*;
        match *self {
            IOError(ref err) => write!(f, "a filesystem related error occured: {}", err),
            ModuleNotFound(ref path) => write!(f, "the specified module could not be found {}", path),
            FunctionNotFoundInModule(ref func, ref path) => write!(f, "unable to locate the function {} in module {}", func, path),
            ParsingError(ref err) => write!(f, "could not parse file: {}", err),
            MalformedAlgoImportSpec(ref path) => write!(f, "encountered malformed algorithm import specification in {}", path),
        }
    }
}

impl From<io::Error> for TypeExtractionError {
    fn from(e: io::Error) -> Self {
        TypeExtractionError::IOError(e)
    }
}

impl From<ParseError> for TypeExtractionError {
    fn from(e: ParseError) -> Self {
        TypeExtractionError::ParsingError(e)
    }
}

impl Error for TypeExtractionError {
    fn description(&self) -> &str {
        // TODO
        ""
    }
}
