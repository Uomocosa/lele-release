pub fn lock_package_block_count(text: &str, name: &str) -> usize {
    text.matches(&format!("\nname = \"{name}\"\n")).count()
}

#[cfg(test)]
mod tests {
    use super::lock_package_block_count;

    #[test]
    fn test_usage() {
        let lock = "version = 4\n\n[[package]]\nname = \"probe\"\nversion = \"0.0.1\"\n\n[[package]]\nname = \"probe\"\nversion = \"0.0.2\"\n";
        assert_eq!(lock_package_block_count(lock, "probe"), 2);
    }
}
