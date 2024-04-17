# branch-dawg

Power tool for managing your git branches.

![Yo Dawg Meme](yodawg.jpg)

## Motivation

You're a great coder and a better team-mate. You want to make it easy for your team review your PRs so you make them as small as possible. You also don't want to block yourself after each PR, so you start chaining branches and PRs together.

Before you know it, the `main` branch moves along and all of your PRs are unmergable. Unbelievable but this is life. Thankfully this dawg is here to help:

```bash
branch-dawg describe
```

Aha! So that's what your chain looks like.

> 📝 TODO: I intend to add automatic chain rebasing, which this section will highlight.

## Usage

To list all branches in a git repository, run:

```bash
branch-dawg list
```

This outputs all local branches.
