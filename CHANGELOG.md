# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Arguments are forwarded to `mask`, so `maski` is a drop-in replacement:
  `maski build --release` runs `mask build --release`, `maski help` runs
  `mask help`.
- Naming a command that only holds subcommands (no script of its own) opens the
  TUI at that level — `maski db` browses `db > migrate, seed`. `←` / `Esc` walk
  back up to the root as usual.
- `examples/maskfile.md`, a sample maskfile exercising subcommands, prompted
  arguments, flags, and the markdown features the preview panel renders.

### Changed

- `maski --help` (and the bare `maski help`) now prints `mask --help`, task list
  included, followed by maski's own options — instead of listing maski's flags
  alone.

### Fixed

- Preview no longer collapses the first item of a list nested in a *tight* list
  item onto its parent's line. mask's `**OPTIONS**` blocks are written that way,
  so every maskfile was affected. The bug is in md4x's ANSI renderer; `build.rs`
  patches a copy of it in `OUT_DIR` until the fix lands upstream.

## [0.1.2] - 2026-06-15

### Fixed

- Section preview now ignores `#` characters inside fenced code blocks. Bash
  comments such as `# 1. step` in a command's script were parsed as level-1
  headings, which cleared the breadcrumb and made the subcommands that followed
  lose their parent prefix — e.g. a `vm > rebuild` section overwrote the
  top-level `rebuild`, so the preview showed the wrong command. The section
  extractor is now fence-aware.

### Changed

- Nix packaging vendors dependencies from `Cargo.lock` (`cargoLock.lockFile`)
  instead of a hardcoded `cargoHash`, so the build no longer breaks on every
  version bump.

## [0.1.1] - 2026-04-14

### Added

- Nix flake (`flake.nix`) and install instructions.
- Cachix CI workflow for prebuilt binaries.
- MIT LICENSE.

### Fixed

- Remove stale git submodule directory before copying the md4x input.
- Force Node.js 24 in GitHub Actions.

## [0.1.0] - 2026-04-11

### Added

- Interactive TUI for `mask` taskfiles, driven by `mask --introspect`.
- Full maskfile section preview rendered to ANSI via the bundled md4x C library.
- Hierarchical navigation of subcommands with skim.
- README with screenshot.

[0.1.2]: https://github.com/ManUtopiK/maski/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ManUtopiK/maski/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ManUtopiK/maski/releases/tag/v0.1.0
