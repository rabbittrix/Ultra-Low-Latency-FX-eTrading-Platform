# Publishing Guide - Crates.io

**Author:** Roberto de Souza <rabbittrix@hotmail.com>  
**License:** Apache-2.0  
**Repository:** <https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform.git>

## 📋 Overview

This guide explains how to publish the Rust crates from this project to [crates.io](https://crates.io), the official Rust package registry.

## Prerequisites

1. **Crates.io Account**: Create an account at <https://crates.io>
2. **API Token**: Generate an API token from your account settings
3. **Cargo Login**: Login to crates.io using `cargo login <token>`
4. **Verify Crate Names**: Check that all crate names are available (see [Crate Naming](#crate-naming) section below)

**⚠️ Important**: This is a **new project** being published for the **first time**. Make sure all crate names are available on crates.io before publishing!

## Publishable Crates

The following crates are designed to be published:

1. **fx-core** - Core matching engine and order book logic
2. **fx-utils** - Shared utilities and common types
3. **fx-md** - Market data processing
4. **fx-pricing** - Pricing engine
5. **fx-risk** - Risk management
6. **fx-router** - Order routing
7. **fx-gateway** - API gateway utilities
8. **fx-proto** - gRPC protocol definitions

## Pre-Publishing Checklist

### 1. Verify Crate Metadata

Ensure each crate's `Cargo.toml` has:

- ✅ Unique name (check availability on crates.io)
- ✅ Version number (semantic versioning)
- ✅ Author information
- ✅ License
- ✅ Repository URL
- ✅ Description
- ✅ Keywords
- ✅ Categories

### 2. Update Version Numbers

```bash
# Check current versions
grep -r "version" crates/*/Cargo.toml

# Update version for new release
# Example: 0.1.0 -> 0.1.1 (patch) or 0.2.0 (minor)
```

### 3. Replace Path Dependencies with Version Requirements

**Important**: Before publishing, you must replace all `path` dependencies with version requirements. Crates.io does not allow `path` dependencies.

**Note**: The automated publishing script (`publish-all.ps1`) handles this automatically. You only need to do this manually if you're publishing crates individually.

#### Manual replacement (if needed)

For each crate, replace:

```toml
# Before (local development)
fx-utils = { path = "../fx-utils" }

# After (for publishing)
fx-utils = { version = "0.1.0" }
```

**Note**: After publishing, you may want to restore path dependencies for local development. Consider using a version control system to manage this.

### 3. Verify Documentation

```bash
# Generate and check documentation
cargo doc --all --no-deps

# Open documentation
cargo doc --open
```

### 4. Run Tests

```bash
# Run all tests
cargo test --all

# Run with verbose output
cargo test --all -- --nocapture
```

### 5. Run Clippy

```bash
# Check for linting issues
cargo clippy --all-targets --all-features -- -D warnings
```

### 6. Format Code

```bash
# Format all code
cargo fmt --all
```

## Publishing Process

### Step 1: Login to Crates.io

```bash
# Login with API token
cargo login <your-api-token>

# Verify login
cargo owner --list
```

### Step 2: Publish All Crates

#### Automated Publishing (Recommended)

**Single command publishes everything automatically:**

```powershell
.\scripts\publish-all.ps1
```

This script will:

1. Check if you're logged in to crates.io
2. Revert all dependencies to path dependencies
3. Publish `fx-utils` first (no dependencies)
4. Update dependencies to version requirements
5. Publish remaining crates in dependency order:
   - **Group 1**: `fx-md`, `fx-risk`, `fx-core` (depend only on `fx-utils`)
   - **Group 2**: `fx-pricing`, `fx-router`, `fx-gateway` (depend on Group 1)
   - **Group 3**: `fx-proto` (depends on `fx-utils`)
6. All automatically - no manual intervention needed

**Note:** The script includes 5-second delays between groups to allow crates.io to index published crates.

#### Manual Publishing (Alternative)

If you prefer to publish manually or need more control, publish crates in dependency order:

##### 1. fx-utils (no dependencies on other project crates)

```bash
cd crates/fx-utils

# Dry run (check without publishing)
cargo publish --dry-run

# Publish
cargo publish
```

**Important**: After publishing `fx-utils`, update dependencies in other crates before publishing them.

#### 2. fx-proto (depends on fx-utils)

```bash
cd crates/fx-proto

# Update dependency to published version
# In Cargo.toml, change:
# fx-utils = { path = "../../crates/fx-utils" }
# to:
# fx-utils = { version = "0.1.0" }

cargo publish --dry-run
cargo publish
```

#### 3. fx-md (depends on fx-utils)

```bash
cd crates/fx-md

# Update dependencies
cargo publish --dry-run
cargo publish
```

#### 4. fx-core (depends on fx-utils)

```bash
cd crates/fx-core

# Update dependencies
cargo publish --dry-run
cargo publish
```

#### 5. fx-risk (depends on fx-utils, fx-core)

```bash
cd crates/fx-risk

# Update dependencies
cargo publish --dry-run
cargo publish
```

#### 6. fx-pricing (depends on fx-utils, fx-risk)

```bash
cd crates/fx-pricing

# Update dependencies
cargo publish --dry-run
cargo publish
```

#### 7. fx-router (depends on fx-utils, fx-core)

```bash
cd crates/fx-router

# Update dependencies
cargo publish --dry-run
cargo publish
```

#### 8. fx-gateway (depends on fx-utils)

```bash
cd crates/fx-gateway

# Update dependencies
cargo publish --dry-run
cargo publish
```

### Step 3: Verify Published Crates

```bash
# Check crate on crates.io
# Visit: https://crates.io/crates/fx-core
# Visit: https://crates.io/crates/fx-utils
# etc.

# Test installation
cargo new test-project
cd test-project
cargo add fx-core --version 0.1.0
cargo build
```

## Version Management

### Semantic Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Updating Versions

```bash
# Update version in Cargo.toml
# Example: version = "0.1.0" -> version = "0.1.1"

# Update workspace version (if applicable)
# In root Cargo.toml: [workspace.package] version = "0.1.1"
```

## Crate Naming

### Current Crate Names

**⚠️ CRITICAL**: Before publishing, verify ALL crate names are available on crates.io!

- `fx-core` - Check: <https://crates.io/crates/fx-core>
- `fx-utils` - Check: <https://crates.io/crates/fx-utils>
- `fx-md` - Check: <https://crates.io/crates/fx-md>
- `fx-pricing` - Check: <https://crates.io/crates/fx-pricing>
- `fx-risk` - Check: <https://crates.io/crates/fx-risk>
- `fx-router` - Check: <https://crates.io/crates/fx-router>
- `fx-gateway` - Check: <https://crates.io/crates/fx-gateway>
- `fx-proto` - Check: <https://crates.io/crates/fx-proto>

### Check Name Availability

**Before publishing, verify each crate name is available:**

```bash
# Check if name is available (returns 404 = available, 200 = taken)
curl https://crates.io/api/v1/crates/fx-core
curl https://crates.io/api/v1/crates/fx-utils
# ... check all crates
```

**Or use PowerShell:**

```powershell
# Check all crate names
$crates = @("fx-core", "fx-utils", "fx-md", "fx-pricing", "fx-risk", "fx-router", "fx-gateway", "fx-proto")
foreach ($crate in $crates) {
    $response = Invoke-WebRequest -Uri "https://crates.io/api/v1/crates/$crate" -UseBasicParsing -ErrorAction SilentlyContinue
    if ($response.StatusCode -eq 200) {
        Write-Host "$crate - ❌ TAKEN" -ForegroundColor Red
    } else {
        Write-Host "$crate - ✅ AVAILABLE" -ForegroundColor Green
    }
}
```

### Alternative Names (if conflicts exist)

If any crate name is taken, consider these alternatives:

- `fx-etrading-core`
- `fx-etrading-utils`
- `fx-etrading-md`
- `fx-etrading-pricing`
- `fx-etrading-risk`
- `fx-etrading-router`
- `fx-etrading-gateway`
- `fx-etrading-proto`

**To rename a crate:**

1. Update the `name` field in the crate's `Cargo.toml`
2. Update any references to the crate in other `Cargo.toml` files
3. Re-run the publishing script

## Publishing Checklist

Before publishing each crate:

- [ ] Version number updated
- [ ] Dependencies updated to published versions
- [ ] Documentation complete (`cargo doc`)
- [ ] Tests passing (`cargo test`)
- [ ] Clippy clean (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] README.md exists (if applicable)
- [ ] License file included
- [ ] Repository URL correct
- [ ] Dry run successful (`cargo publish --dry-run`)

## Post-Publishing

### 1. Update Documentation

Update `README.md` and other docs to reference published crates:

```toml
# Instead of:
fx-core = { path = "../../crates/fx-core" }

# Use:
fx-core = { version = "0.1.0" }
```

### 2. Tag Release

```bash
# Create git tag
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0
```

### 3. Create GitHub Release

1. Go to GitHub repository
2. Click "Releases" → "Create a new release"
3. Tag: `v0.1.0`
4. Title: "Release v0.1.0"
5. Description: List of changes
6. Publish release

## Troubleshooting

### Name Already Taken

If crate name is taken, choose alternative:

```toml
# Option 1: Add prefix
name = "fx-etrading-core"

# Option 2: Add suffix
name = "fx-core-etrading"

# Option 3: Use different name
name = "ultra-low-latency-fx-core"
```

### Dependency Not Found

If dependency isn't published yet:

1. Publish dependencies first
2. Wait a few minutes for crates.io to index
3. Try publishing again

### Authentication Errors

```bash
# Re-login
cargo logout
cargo login <new-token>
```

### Version Already Published

If version already exists:

1. Bump version number
2. Update `Cargo.toml`
3. Publish again

## Best Practices

1. **Start Small**: Publish one crate first to test the process
2. **Use Dry Run**: Always test with `--dry-run` first
3. **Version Carefully**: Follow semantic versioning
4. **Document Well**: Good documentation increases adoption
5. **Test Installation**: Verify crates can be installed
6. **Monitor Issues**: Check crates.io for user feedback

## Example: Publishing fx-utils

```bash
# 1. Navigate to crate
cd crates/fx-utils

# 2. Verify Cargo.toml
cat Cargo.toml

# 3. Run tests
cargo test

# 4. Check documentation
cargo doc --no-deps

# 5. Dry run
cargo publish --dry-run

# 6. Publish
cargo publish

# 7. Verify on crates.io
# Visit: https://crates.io/crates/fx-utils
```

## Support

For publishing issues:

- **Crates.io Documentation**: <https://doc.rust-lang.org/cargo/reference/publishing.html>
- **GitHub Issues**: <https://github.com/rabbittrix/Ultra-Low-Latency-FX-eTrading-Platform/issues>
- **Email**: <rabbittrix@hotmail.com>
