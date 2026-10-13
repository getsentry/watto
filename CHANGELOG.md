# Changelog

## 0.3.0

### Bug Fixes 🐛

- (tests) Verify alignment by @loewenheim in [#13](https://github.com/getsentry/watto/pull/13)
- Bounds-check untrusted entry lengths in OffsetSet reads by @paodingo in [#23](https://github.com/getsentry/watto/pull/23)

### Internal Changes 🔧

- (ci) Pin macOS runner to macos-15 by @tobias-wilfert in [#19](https://github.com/getsentry/watto/pull/19)
- (codeowners) Update codeowners by @tobias-wilfert in [#21](https://github.com/getsentry/watto/pull/21)
- (release) Switch from action-prepare-release to Craft by @BYK in [#17](https://github.com/getsentry/watto/pull/17)
- Pin GitHub Actions to full-length commit SHAs by @joshuarli in [#18](https://github.com/getsentry/watto/pull/18)

### Other

- Add `OffsetSet` as a more generic version of `StringTable` by @Swatinem in [#14](https://github.com/getsentry/watto/pull/14)

## 0.2.0

### Various fixes & improvements

- Reimplement `StringTable` in terms of `hashbrown::HashTable` (#12) by @Swatinem
- Add `StringTable::from_bytes` to deserialize a `StringTable` directly from its buffer. (#11) by @Swatinem
- ci: Actually run doctests (#10) by @Swatinem
- feat: Add align_to_type functions (#9) by @loewenheim

## 0.1.0

Inception

