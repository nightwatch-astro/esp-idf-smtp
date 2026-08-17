# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/nightwatch-astro/esp-idf-smtp/compare/v0.4.0...v0.4.1) - 2026-07-17

### Miscellaneous

- complete crates.io metadata (documentation/homepage) ([#25](https://github.com/nightwatch-astro/esp-idf-smtp/pull/25))

### Ci

- *(cla)* don't lock PRs after merge ([#27](https://github.com/nightwatch-astro/esp-idf-smtp/pull/27))

## [0.4.0](https://github.com/nightwatch-astro/esp-idf-smtp/compare/v0.3.1...v0.4.0) - 2026-07-14

### Bug Fixes

- use GitHub App token for CLA bot ([#23](https://github.com/nightwatch-astro/esp-idf-smtp/pull/23))

### Miscellaneous

- [**breaking**] relicense from Apache-2.0 to MPL-2.0 ([#22](https://github.com/nightwatch-astro/esp-idf-smtp/pull/22))
- exclude shell and tooling scaffolding from language stats
- *(deps)* bump dorny/paths-filter from 4.0.1 to 4.0.2 in the minor-and-patch group ([#19](https://github.com/nightwatch-astro/esp-idf-smtp/pull/19))
- *(deps)* bump actions/checkout from 6.0.2 to 6.0.3 in the minor-and-patch group ([#17](https://github.com/nightwatch-astro/esp-idf-smtp/pull/17))
- *(deps)* bump the minor-and-patch group with 2 updates ([#16](https://github.com/nightwatch-astro/esp-idf-smtp/pull/16))

### Ci

- bump shared release workflow to App-token version ([#20](https://github.com/nightwatch-astro/esp-idf-smtp/pull/20))

## [0.3.1](https://github.com/nightwatch-astro/esp-idf-smtp/compare/v0.3.0...v0.3.1) - 2026-04-05

### Miscellaneous

- enable clippy pedantic lints and ESP-IDF 5.5 compat ([#14](https://github.com/nightwatch-astro/esp-idf-smtp/pull/14))

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
