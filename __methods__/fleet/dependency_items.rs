use toml_edit::{DocumentMut, Item};

pub fn dependency_items(document: &DocumentMut) -> Vec<(String, &Item)> {
    let mut items = Vec::new();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = document.get(section).and_then(Item::as_table_like) {
            for (key, item) in table.iter() {
                items.push((key.to_string(), item));
            }
        }
    }
    if let Some(table) = document
        .get("workspace")
        .and_then(Item::as_table_like)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Item::as_table_like)
    {
        for (key, item) in table.iter() {
            items.push((key.to_string(), item));
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use toml_edit::DocumentMut;

    use super::dependency_items;

    #[test]
    fn test_usage() {
        let document: DocumentMut =
            "[dependencies]\nalpha = \"1\"\n\n[workspace.dependencies]\nbeta = \"2\"\n"
                .parse()
                .unwrap();

        let items = dependency_items(&document);

        let names: Vec<&str> = items.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "beta"]);
    }
}
