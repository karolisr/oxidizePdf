# Contributing to oxidizePdf

Thank you for your interest in contributing to oxidizePdf! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites
- Rust 1.70+ (stable)
- Git
- A GitHub account

### Development Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/oxidizePdf.git
   cd oxidizePdf
   ```

2. **Install Dependencies**
   ```bash
   cargo build --workspace
   ```

3. **Run Tests**
   ```bash
   cargo test --workspace
   ```

## Development Workflow

### Before Making Changes

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Ensure Clean Starting Point**
   ```bash
   cargo fmt --all
   cargo clippy --all -- -D warnings
   cargo test --workspace
   ```

### Making Changes

1. **Follow Development Guidelines**
   - See [docs/DEVELOPMENT_GUIDELINES.md](docs/DEVELOPMENT_GUIDELINES.md) for detailed patterns
   - Use `cargo fmt --all` for consistent formatting
   - Address all clippy warnings: `cargo clippy --all -- -D warnings`

2. **Write Tests**
   - Add unit tests for new functionality
   - Update integration tests if needed
   - Ensure all tests pass: `cargo test --workspace`

3. **Document Changes**
   - Update public API documentation
   - Add examples for new features
   - Update CHANGELOG.md for significant changes

### Pre-commit Validation

**Required checks before committing:**

```bash
# 1. Format code
cargo fmt --all

# 2. Check for warnings (treat as errors)
cargo clippy --all -- -D warnings

# 3. Run all tests
cargo test --workspace

# 4. Build entire workspace
cargo build --workspace
```

### Submitting Changes

1. **Commit with Descriptive Messages**
   ```bash
   git commit -m "feat: add PDF merge functionality
   
   - Implement MergeOptions for customization
   - Add comprehensive tests
   - Update API documentation"
   ```

2. **Push to Your Fork**
   ```bash
   git push origin feature/your-feature-name
   ```

3. **Create Pull Request**
   - Use the GitHub web interface
   - Fill out the PR template
   - Link related issues

## GitFlow Workflow

This project follows a strict GitFlow workflow. Understanding and following this workflow is **mandatory** for all contributions.

### Branch Structure

```
main (production-ready code)
│
└── development (integration branch)
    │
    ├── feature/* (new features)
    ├── release/* (release preparation)
    └── hotfix/* (critical production fixes)
```

### GitFlow Rules

#### 1. Feature Development

**All new features MUST:**
- Be created from `development` branch
- Be merged back to `development` branch
- NEVER be merged directly to `main`

```bash
# Create feature branch
git checkout development
git pull origin development
git checkout -b feature/new-feature

# Work on feature
git add .
git commit -m "feat: implement new feature"

# Merge back to development
git checkout development
git merge feature/new-feature
git push origin development
```

#### 2. Release Process

**Releases are created from `development` and merged to BOTH `main` and `development`:**

```bash
# Create release branch
git checkout development
git checkout -b release/v1.2.0

# Bump version, final fixes
# Update CHANGELOG.md
git commit -m "chore: prepare release v1.2.0"

# Merge to main
git checkout main
git merge --no-ff release/v1.2.0
git tag v1.2.0
git push origin main --tags

# Merge back to development
git checkout development
git merge --no-ff release/v1.2.0
git push origin development

# Delete release branch
git branch -d release/v1.2.0
```

#### 3. Hotfix Process

**Hotfixes are the ONLY branches that can be created from `main`:**

```bash
# Create hotfix from main
git checkout main
git checkout -b hotfix/critical-bug

# Fix the bug
git commit -m "fix: resolve critical production issue"

# Merge to main
git checkout main
git merge --no-ff hotfix/critical-bug
git tag v1.1.1
git push origin main --tags

# IMMEDIATELY merge to development
git checkout development
git merge --no-ff hotfix/critical-bug
git push origin development

# Delete hotfix branch
git branch -d hotfix/critical-bug
```

### Critical Rules

1. **NEVER create feature branches from tags or `main`**
2. **NEVER merge features directly to `main`**
3. **ALWAYS ensure `development` contains everything in `main`**
4. **Hotfixes MUST be merged to both `main` AND `development`**

### Common Mistakes to Avoid

❌ **Wrong:**
```bash
# Creating feature from main or tag
git checkout main
git checkout -b feature/new-feature

# Merging feature directly to main
git checkout main
git merge feature/new-feature
```

✅ **Correct:**
```bash
# Always from development
git checkout development
git checkout -b feature/new-feature

# Always back to development
git checkout development
git merge feature/new-feature
```

### Branch Naming Conventions

- Features: `feature/descriptive-name`
- Releases: `release/vX.Y.Z`
- Hotfixes: `hotfix/critical-issue-name`

### Pull Request Guidelines for GitFlow

1. **Feature PRs**: Target `development` branch
2. **Release PRs**: Create two PRs - one to `main`, one to `development`
3. **Hotfix PRs**: Create two PRs - one to `main`, one to `development`

## CI/CD Pipeline

### Automated Checks
Our CI/CD pipeline runs on every PR and includes:
- **Format Check**: `cargo fmt --check`
- **Clippy Linting**: `cargo clippy --all -- -D warnings`
- **Build Verification**: `cargo build --workspace`
- **Test Suite**: `cargo test --workspace`
- **Code Coverage**: Coverage analysis and reporting

### Troubleshooting Failed Pipelines

#### Format Failures
```bash
# Fix formatting issues
cargo fmt --all

# Verify locally
cargo fmt --all -- --check
```

#### Clippy Failures
Common patterns and fixes are documented in [docs/DEVELOPMENT_GUIDELINES.md](docs/DEVELOPMENT_GUIDELINES.md).

Quick fixes for common issues:
```bash
# Modern I/O errors
std::io::Error::other("message")  // Instead of Error::new(ErrorKind::Other, "message")

# Avoid unnecessary clones
map.insert(*id, value)  // Instead of map.insert(id.clone(), value)

# Prefer value iteration
for value in map.values()  // Instead of for (_, value) in &map

# Use is_empty()
!vec.is_empty()  // Instead of vec.len() > 0
```

#### Build Failures
- Check for missing dependencies in `Cargo.toml`
- Verify feature flags are correctly configured
- Ensure cross-platform compatibility

#### Test Failures
```bash
# Run specific test
cargo test test_name -- --nocapture

# Run tests with output
cargo test -- --nocapture

# Run only integration tests
cargo test --test integration_test_name
```

## Code Style and Standards

### Rust Guidelines
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` default configuration
- Address all clippy warnings
- Prefer explicit error handling over `unwrap()`

### Documentation
- Document all public APIs with `///` comments
- Include examples in documentation comments
- Use `cargo doc --open` to verify documentation builds
- Keep examples simple and focused

### Testing
- Write unit tests for all new functionality
- Add integration tests for complex workflows
- Use descriptive test names: `test_merge_preserves_bookmarks`
- Mock external dependencies appropriately

## Community Guidelines

### Code of Conduct
- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow GitHub's community guidelines

### Communication
- **Issues**: Report bugs and request features
- **Discussions**: Ask questions and share ideas
- **PRs**: Submit code changes
- **Email**: For security issues only

## Types of Contributions

### 🐛 Bug Reports
- Use the bug report template
- Include minimal reproduction steps
- Provide system information
- Check for existing reports first

### ✨ Feature Requests
- Use the feature request template
- Explain the use case
- Consider backwards compatibility
- Discuss design before implementation

### 📚 Documentation
- Improve API documentation
- Add usage examples
- Fix typos and grammar
- Translate documentation

### 🧪 Testing
- Add missing test coverage
- Improve test reliability
- Create property-based tests
- Add performance benchmarks

## Project Structure

```
oxidizePdf/
├── oxidize-pdf-core/     # Core PDF library
├── oxidize-pdf-cli/      # Command-line interface
├── oxidize-pdf-api/      # REST API server
├── test-suite/           # Integration tests
├── docs/                 # Documentation
├── .github/              # GitHub workflows
└── README.md
```

## Getting Help

### Resources
- [Development Guidelines](docs/DEVELOPMENT_GUIDELINES.md)
- [API Documentation](https://docs.rs/oxidize-pdf)
- [Rust Documentation](https://doc.rust-lang.org/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)

### Support Channels
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and community discussion
- **Documentation**: In-code examples and API docs

## License

By contributing to oxidizePdf, you agree that your contributions will be licensed under the same license as the project (GPL v3 for Community Edition).

## Recognition

Contributors are recognized in:
- CHANGELOG.md for significant contributions
- GitHub contributors list
- Release notes for major features

---

Thank you for contributing to oxidizePdf! Your efforts help make PDF processing in Rust better for everyone.