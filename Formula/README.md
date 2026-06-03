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
2. **Tag releases** — create a `v0.3.0` tag in both repos:
   - `interesting-vibe-coding/paws`
   - `MisterBrookT/paws-games`
3. **Fill in sha256** — download each tarball and run `shasum -a 256 <file>`,
   then paste the hash into the corresponding formula's `sha256` field.
4. **Push the tap repo** — once formulae have valid sha256 values, the stable
   `brew install paws` path is live.

Until steps 1–4 are done, only `--HEAD` installs work.
