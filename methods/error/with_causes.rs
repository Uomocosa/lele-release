use crate::Error;

#[must_use]
pub fn with_causes(error: &Error) -> String {
    let mut message = error.to_string();
    let mut cause = std::error::Error::source(error);
    while let Some(inner) = cause {
        message.push_str(": ");
        message.push_str(&inner.to_string());
        cause = inner.source();
    }
    message
}

#[cfg(test)]
mod tests {
    use super::with_causes;
    use crate::Error;

    #[test]
    fn test_usage() {
        let error = Error::Io {
            context: String::from("stdin"),
            source: std::io::Error::other("closed"),
        };
        assert_eq!(with_causes(&error), "stdin: closed");
    }
}
