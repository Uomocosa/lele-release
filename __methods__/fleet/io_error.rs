use std::path::Path;

use crate::Error;

pub fn io_error(path: &Path, source: std::io::Error) -> Error {
    Error::Io {
        context: path.display().to_string(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::io_error;
    use crate::Error;

    #[test]
    fn test_usage() {
        let error = io_error(Path::new("Cargo.toml"), std::io::Error::other("gone"));
        assert!(matches!(&error, Error::Io { context, .. } if context == "Cargo.toml"));
    }
}
