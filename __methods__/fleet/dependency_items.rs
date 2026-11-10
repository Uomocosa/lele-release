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
    use super::dependency_items;

    #[test]
    fn test_usage() {
        let _ = dependency_items;
    }
}
