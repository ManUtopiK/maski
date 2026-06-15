# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
