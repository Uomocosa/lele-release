/// Git exports these when it runs a clean/smudge filter, so a child process that keeps them
/// resolves every path against the filter's repository instead of the directory it was given.
pub const GIT_REPO_LOCATING_ENV_VAR_NAMES: [&str; 8] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_PREFIX",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_NAMESPACE",
];
