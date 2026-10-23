#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bump {
    Patch,
    Minor,
    Major,
}

#[derive(Clone, Debug, Default)]
pub enum Selection {
    #[default]
    All,
    Only(Vec<String>),
    Exclude(Vec<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinsMode {
    Clean,
    Smudge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PinForm {
    Git(String),
    Tag(String),
    Rev(String),
    Branch(String),
    Path(String),
}
