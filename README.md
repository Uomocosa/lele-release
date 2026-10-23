# lele-release

One command releases the whole fleet:

```text
just release patch      # 0.0.8 -> 0.0.9
just release minor      # 0.0.8 -> 0.1.0
just release major      # 0.0.8 -> 1.0.0
```

The fleet is a set of separate repositories that depend on each other. They are
cloned separately, so every dependency in a *commit* is an immutable git pin:

```toml
atomic-delegate-macros = { git = "https://github.com/Uomocosa/lele-rust-linter", tag = "v0.0.4" }
```

But while you work, the same manifest in your working tree holds a sibling path:

```toml
atomic-delegate-macros = { path = "../lele-rust-linter/crates/atomic-delegate-macros" }
```

A git content filter keeps both forms in sync automatically: commits carry pins,
the working tree carries paths. Editing a dependency is picked up by the next
`cargo build` in every dependent, with no commands.

## Install

```bash
cd lele-release
just install          # cargo install --locked --force --path .
just setup            # install the git filter for this machine
just hooks            # install the prek commit/push gates
just tools            # verify the installed version
```

`just setup` writes two things, both machine-local:

| File | Content |
|---|---|
| `~/.config/git/lele-pins.gitconfig` | the `filter.lele-pins.clean`/`smudge` commands, rendered from `pins.gitconfig.template` with the absolute binary path |
| `~/.gitconfig` | one `includeIf "hasconfig:remote.*.url:https://github.com/Uomocosa/**"` pointing at the file above |

It then prints which repos still miss the committed attribute line. Add it once
per repo (and commit it):

```gitattributes
Cargo.toml filter=lele-pins
```

`git config --unset` the filter (or delete the included file) to switch the
machine back to plain pin behaviour; nothing in the repos changes.

## How development works

```text
                     working tree                  commit / clone
Cargo.toml    path = "../lele-rust-linter/…"   tag = "v0.0.4"
                     ^          smudge                 ^
                     |  (checkout)                     |  clean (git add)
                     +---------------------------------+
```

- **smudge** runs on checkout: `git` pins become `path` pins when the sibling
  checkout exists. If it does not exist (plain clone, CI), the file is passed
  through unchanged, which is exactly the online form.
- **clean** runs on `git add`: a `path` that points into another fleet repo
  becomes a `git` pin at that repo's latest tag, or a `rev` of `origin/main`
  while the dependency has unreleased commits.
- Because git compares `clean(worktree)` with the index, `git status` stays
  clean even though the working tree differs from the commit.

Verify a repo:

```bash
git check-attr filter -- Cargo.toml     # Cargo.toml: filter: lele-pins
git config --get filter.lele-pins.clean # /home/…/lele-release pins clean %f
grep 'path = ' Cargo.toml               # working tree form
git cat-file -p HEAD:Cargo.toml | grep tag   # committed form
cargo tree -i lele-bevy-preview         # shows a path source
```

A machine without the filter simply checks out pins and behaves like CI. The
filter never fails a git operation: on any error it passes the content through.

## Commands

| Command | What it does |
|---|---|
| `lele-release status` | fleet table: version, tag, commits since tag, tree state, pins, filter state, what a release would do |
| `lele-release release <patch\|minor\|major>` | preflight → bump → pin → commit → tag → atomic push, in dependency order |
| `--only a,b` / `--exclude c` | restrict the release set (mutually exclusive); a full run is the default |
| `--dry-run` | print the plan and the affected set, write nothing |
| `lele-release setup` | install the git filter for this machine |
| `lele-release pins clean\|smudge` | internal, invoked by git; hidden from help |

Exit codes: `0` success or nothing to do, `1` failure (preflight, hook, push).

`status` and `release` are wrapped by an uncommitted `justfile` at the fleet
root:

```make
status:
    lele-release --root . status

release level *args:
    lele-release --root . release {{level}} {{args}}
```

## Release semantics

```text
changed(r) = commits since r's latest tag > 0          (no tag at all also counts)
stale(r)   = a pin in r does not point at its dependency's latest tag
affected   = changed ∪ every transitive dependent of a changed repo
```

- A dependency change forces its dependents to re-pin, and a re-pin is a
  manifest change, so dependents are released in the same run. After a full run
  `changed` and `stale` are both empty, and the next run prints
  `nothing to release`. No repo is ever left behind by accident.
- `--only`/`--exclude` deliberately leave work pending; `status` shows it and
  the next full run picks it up.
- Order is dependency-first (Kahn). A cycle is a refusal, never a guess.

Per repo, in order:

```text
1. fetch · clean tree · on main · HEAD == origin/main · version == latest tag
2. bump the version in [package] or [workspace.package]
3. normalise Cargo.lock: every fleet dependency entry gets the git source of
   its dependency's new tag (offline; the SHA comes from the tag itself)
4. git add (the version file + Cargo.lock) · commit "chore(release): X.Y.Z"
   — the clean filter writes the pins here
5. git tag -a vX.Y.Z
6. git push --atomic origin main vX.Y.Z          (pre-push hooks run here)
7. on rejection: reset --hard origin/main and delete the local tag
```

There is no ledger and no `--resume`: a repo is "done" when its version equals
its tag and it has no commits since it. Re-running after an interruption skips
finished repos and continues with the rest. Because the push is atomic, a repo
is never left half-released.

The tool never force-pushes, never moves a tag, and never rebases.

## Pin forms

| Situation | Committed form |
|---|---|
| dependency released | `tag = "vX.Y.Z"` |
| dependency has unreleased commits | `rev = "<origin/main sha>"` |
| same repo (workspace member) | `path = "crates/…"` |
| local development | `path = "../sibling/…"` (smudged, never committed) |

`branch = "main"` is not used: it makes builds depend on when `cargo update` ran
instead of on what you released.

## Fleet map

```text
lele-rust-linter            atomic-delegate-macros, lele-lint, lele-bevy-lint, lele-snake-case
lele-rust-function-taxonomy atomic-delegate-macros
telegram-bot-helper         atomic-delegate-macros; telegram-bot-helper, telegram-bot-helper-macros, lele-bevy-preview
lele-release                atomic-delegate-macros
freenet-contract-harness    —
freenet-libp2p-bevy-plugin  atomic-delegate-macros, telegram-bot-helper, lele-bevy-preview, freenet-contract-harness
```

One version and one `vX.Y.Z` tag per repo, including repos with several crates
(the workspace version is the release version).

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `Cargo.toml` shows `tag = …` in the working tree | filter not installed, or the sibling checkout is missing: `just setup` |
| `git status` clean but the file has paths | correct: the filter cleans on read |
| `Cargo.lock` dirty after a build | expected in path mode; `release` restores and normalises it |
| `release` refuses: working tree not clean | commit or stash first; the tool only creates the release commit |
| `release` refuses: version does not match tag | the repo drifted: bump the manifest to the tag, or release it from a clean state |
| `release` refuses: not in sync | push or pull; the tool never releases an unpushed tree |
| `release` stopped midway | fix the failure and run the same command; finished repos are skipped |
| `--only` left repos behind | intended; the next full release converges |
| filter silently missing after moving the workspace | the include file is gone; re-run `just setup` |
| push rejected by a hook | the release rolled back locally; fix the hook failure and re-run |

## Design notes

- **Not a monorepo, not a registry.** The repos must clone independently, so the
  committed form has to be a remote source. Local folders are a *working tree*
  concern, which is exactly what a git content filter expresses.
- **Not `path` in commits.** A clone outside the folder layout cannot resolve it.
- **Not a push hook that rewrites manifests.** A hook can accept or reject a
  push; it cannot change the commits being pushed without changing their SHAs.
- **Not `branch = "main"`.** Then the consumer's `Cargo.lock` is the only pin,
  and "latest" depends on when someone last ran `cargo update`.
- **Why pins point at tags:** readable in diffs, immutable by policy, and the
  dependency graph is visible on GitHub. The tool verifies each tag resolves.

## Hacking on the tool

```bash
just build      # cargo build --all-targets --all-features
just test       # cargo nextest run --all-targets --all-features
just clippy     # -D warnings (pedantic + nursery)
just lint       # lele-lint .
```

Layout follows the fleet conventions: one type per `src/` file, method bodies in
`methods/<type>/<method>.rs`, `__basic__/` for behaviour-free types.

- `src/fleet.rs` — the repo graph and the public operations
- `src/repo.rs` — one git repo: tags, readiness, the release sequence
- `methods/fleet/pins.rs` — read pins out of manifests
- `methods/fleet/clean_manifest.rs` / `smudge_manifest.rs` — the filter transforms
- `methods/fleet/plan.rs` — changed/affected/order
