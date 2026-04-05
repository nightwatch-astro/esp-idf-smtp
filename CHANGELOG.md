# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/nightwatch-astro/esp-idf-smtp/compare/v0.2.2...v0.3.0) - 2026-04-05

### Features

- add CI OK gate job for branch protection ([#11](https://github.com/nightwatch-astro/esp-idf-smtp/pull/11))

### Miscellaneous

- pin GitHub Actions to commit SHAs
- pin GitHub Actions to commit SHAs
- pin GitHub Actions to commit SHAs
- add CODEOWNERS for CI security

### Performance

- *(ci)* use nextest and clippy --all-targets ([#13](https://github.com/nightwatch-astro/esp-idf-smtp/pull/13))
- *(ci)* replace rust-cache with sccache ([#12](https://github.com/nightwatch-astro/esp-idf-smtp/pull/12))

## [0.2.2](https://github.com/nightwatch-astro/esp-idf-smtp/compare/v0.2.1...v0.2.2) - 2026-04-01

### Bug Fixes

- ESP-IDF 5.5 compatibility ([#9](https://github.com/nightwatch-astro/esp-idf-smtp/pull/9))

### Miscellaneous

- add pre-commit config with Rust hooks ([#7](https://github.com/nightwatch-astro/esp-idf-smtp/pull/7))

## [0.2.1](https://github.com/nightwatch-astro/esp-idf-smtp/compare/v0.2.0...v0.2.1) - 2026-03-30

### Bug Fixes

- *(ci)* remove semver-check job for ESP-IDF embedded crate
- *(ci)* restore dependabot config with grouping

### Miscellaneous

- *(deps)* bump dorny/paths-filter from 3 to 4 ([#6](https://github.com/nightwatch-astro/esp-idf-smtp/pull/6))

### Ci

- add minor+patch grouping to dependabot

## [0.2.0](https://github.com/nightwatch-astro/esp-idf-smtp/compare/v0.1.0...v0.2.0) - 2026-03-29

### Features

- *(ci)* add release environment to publish job

### Miscellaneous

- add .gitattributes for linguist-generated patterns

### Refactoring

- *(ci)* use shared reusable release workflow

### Ci

- skip semver-check when no Rust code changes
- auto-merge minor dependency updates
