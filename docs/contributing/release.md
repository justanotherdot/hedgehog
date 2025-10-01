# release process

## prerequisites

Install `cargo-workspaces`:

```sh
cargo install cargo-workspaces
```

Ensure you have a `CARGO_TOKEN` secret configured in GitHub repository settings with your crates.io API token.

## pre-release checks

Run all checks before creating a release:

```sh
# ensure workspace compiles cleanly
cargo check --workspace

# run tests
cargo test --workspace

# run clippy on library code
cargo clippy --workspace --lib --all-features -- -D warnings

# verify documentation builds
cargo doc --workspace --no-deps

# ensure formatting is consistent
cargo fmt --all -- --check
```

All checks must pass before proceeding.

## version bump

Bump version across all workspace crates:

```sh
# for patch releases (0.1.0 -> 0.1.1)
cargo workspaces version patch --yes

# for minor releases (0.1.0 -> 0.2.0)
cargo workspaces version minor --yes

# for major releases (0.1.0 -> 1.0.0)
cargo workspaces version major --yes
```

This command:
- Updates version in all `Cargo.toml` files
- Updates inter-workspace dependencies
- Creates a git commit
- Creates a git tag (e.g., `v0.1.1`)

## push release

Push the version bump commit and tags:

```sh
git push --follow-tags
```

GitHub Actions will automatically:
1. Run CI checks
2. Publish all workspace crates to crates.io in dependency order
3. Create a GitHub release

## manual publishing

If automated publishing fails or you need to publish manually:

```sh
# publish in dependency order
cd hedgehog-core && cargo publish
cd ../hedgehog-derive && cargo publish
cd ../hedgehog && cargo publish
```

Wait a few minutes between each publish for crates.io to update its index.

## post-release

Verify the release:
- Check crates.io pages for all three crates
- Verify documentation on docs.rs
- Test installation in a fresh project

## troubleshooting

**Publishing fails with dependency errors:**
Wait 2-3 minutes for crates.io index to update between crate publishes.

**Version already exists:**
Delete the git tag locally and remotely, fix the issue, then re-run version bump:
```sh
git tag -d v0.1.1
git push origin :refs/tags/v0.1.1
```

**CI failing:**
Fix the issue, then push fixes. If the tag was already created, you'll need to delete it (see above) and re-release.
