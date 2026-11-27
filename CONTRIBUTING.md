# Contributing to Rovarch

## Working on the code

Rovarch is a Rust application built on GTK4 and libadwaita. Read
[AGENTS.md](AGENTS.md) for the layout rules before touching the tree.

A healthy change:

- compiles, and passes `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo test`
- keeps the code self-explanatory, with no comments that restate the code
- is green on the `check` workflow in CI

## Commit messages

Write a short imperative line in the present tense, with no prefix:

```
Fix region list for Chile
Remove telemetry bar from the Play page
Bump to v0.4.0
```

Add a body when the reasoning matters. The release workflow builds release
notes from the commit log, so a clear first line ends up on the public
release page.

## Artificial intelligence

AI assistance is allowed and common here. See
[AI_USAGE_POLICY.md](AI_USAGE_POLICY.md) for the rules on authorship and
review.
