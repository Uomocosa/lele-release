use toml_edit::Item;

use crate::basic;

pub fn pin_form(table: &dyn toml_edit::TableLike) -> Option<basic::enums::PinForm> {
    if let Some(path) = table.get("path").and_then(Item::as_str) {
        return Some(basic::enums::PinForm::Path(path.to_string()));
    }
    let url = table.get("git").and_then(Item::as_str)?.to_string();
    if let Some(tag) = table.get("tag").and_then(Item::as_str) {
        return Some(basic::enums::PinForm::Tag(tag.to_string()));
    }
    if let Some(rev) = table.get("rev").and_then(Item::as_str) {
        return Some(basic::enums::PinForm::Rev(rev.to_string()));
    }
    if let Some(branch) = table.get("branch").and_then(Item::as_str) {
        return Some(basic::enums::PinForm::Branch(branch.to_string()));
    }
    Some(basic::enums::PinForm::Git(url))
}

#[cfg(test)]
mod tests {
    use toml_edit::DocumentMut;
    use toml_edit::Item;

    use super::pin_form;
    use crate::basic;

    #[test]
    fn test_usage() {
        let document: DocumentMut =
            "[dep]\ngit = \"https://example.com/dep.git\"\ntag = \"v0.0.2\"\n"
                .parse()
                .unwrap();
        let table = document.get("dep").and_then(Item::as_table_like).unwrap();

        let form = pin_form(table);

        assert_eq!(
            form,
            Some(basic::enums::PinForm::Tag(String::from("v0.0.2")))
        );
    }
}
