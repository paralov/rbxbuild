# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-10-16

### Added
- Initial release of rojo-build-lite
- Convert Rojo project JSON to Roblox XML format
- Support for DataModel (place files) and Model files
- Automatic service class inference (Workspace, ReplicatedStorage, etc.)
- Property type resolution (Vector3, Color3, CFrame, enums, etc.)
- Comprehensive test suite covering JSON to XML conversion
- CI/CD pipeline with GitHub Actions
- Automatic binary releases for Linux, macOS, and Windows

### Fixed
- Fixed duplicate Name property issue when explicitly provided in $properties
- Name property now correctly uses explicit override when provided

[Unreleased]: https://github.com/paralov/rbxbuild/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/paralov/rbxbuild/releases/tag/v0.1.0
