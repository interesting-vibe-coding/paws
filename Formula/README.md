# Homebrew Formulae for Paws

This directory contains Homebrew formulae for installing Paws and its game library.

## Install (once live)

```bash
brew tap interesting-vibe-coding/paws
brew install paws        # the paws terminal pet
brew install paws-games  # all three games (jump-high, earth-online, tetris)
```

## What works TODAY (`--HEAD`)

The `--HEAD` variant builds from the latest `main` branch — no release tag needed:

```bash
brew install --HEAD interesting-vibe-coding/paws/paws
brew install --HEAD interesting-vibe-coding/paws/paws-games
```

## How Homebrew taps work

`brew tap interesting-vibe-coding/paws` maps to the GitHub repo
**`interesting-vibe-coding/homebrew-paws`**. Homebrew looks for formulae
inside `Formula/` in that tap repo.

## Remaining steps to make stable install live

1. **Create the tap repo** — `interesting-vibe-coding/homebrew-paws` on GitHub.
   Copy `paws.rb` and `paws-games.rb` into its `Formula/` directory.
2. **Tags** — `v0.3.1` has been created in `interesting-vibe-coding/paws`.
   The release workflow (`.github/workflows/release.yml`) triggers on `v*` tag
   pushes and will produce pre-built binaries attached to the GitHub Release.
   `paws-games` still needs its own tag once that repo is ready.
3. **sha256** — both formulae already have sha256 values filled in.
   If the release workflow rebuilds and re-uploads tarballs the hashes may need
   to be re-verified with `shasum -a 256 <downloaded-tarball>`.
4. **Push the tap repo** — once the tap repo exists and formulae are copied in,
   the stable `brew install paws` path is live.

Until steps 1–4 are done, only `--HEAD` installs work.
