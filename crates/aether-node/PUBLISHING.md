# Publishing to NPM

This guide explains how to publish `@aether/codegen` to NPM.

## Prerequisites

1. **NPM Account**: Create an account at [npmjs.com](https://www.npmjs.com/)
2. **NPM Token**: Generate an access token from your NPM account settings
3. **Rust Toolchain**: Ensure Rust is installed with the required targets

## Automated Publishing (Recommended)

The project uses GitHub Actions for automated cross-platform builds and publishing.

### Setup

1. Add your NPM token as a GitHub secret:
   - Go to your repository → Settings → Secrets and variables → Actions
   - Create a new secret named `NPM_TOKEN`
   - Paste your NPM access token

2. Create a new release:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. GitHub Actions will automatically:
   - Build native binaries for all platforms (Windows, macOS, Linux)
   - Run tests
   - Publish to NPM

## Manual Publishing

### 1. Build for Current Platform

```bash
cd crates/aether-node
npm install
npm run build
```

### 2. Login to NPM

```bash
npm login
```

### 3. Publish

```bash
npm publish --access public
```

### Cross-Platform Builds

For cross-platform support, you need to build on each target platform or use cross-compilation:

#### Windows (x64)
```bash
npm run build -- --target x86_64-pc-windows-msvc
```

#### macOS (Intel)
```bash
npm run build -- --target x86_64-apple-darwin
```

#### macOS (Apple Silicon)
```bash
npm run build -- --target aarch64-apple-darwin
```

#### Linux (x64)
```bash
npm run build -- --target x86_64-unknown-linux-gnu
```

## Package Structure

After building, your package should contain:

```
crates/aether-node/
├── package.json
├── index.js
├── index.d.ts
├── aether-codegen.win32-x64-msvc.node    # Windows x64
├── aether-codegen.darwin-arm64.node       # macOS ARM64
├── aether-codegen.darwin-x64.node         # macOS x64
├── aether-codegen.linux-x64-gnu.node      # Linux x64
└── ...
```

## Scoped Package

The package is published under the `@aether` scope. If you don't own this scope on NPM, you'll need to:

1. Change the package name in `package.json`:
   ```json
   {
     "name": "@your-username/aether-codegen"
   }
   ```

2. Or publish without a scope:
   ```json
   {
     "name": "aether-codegen"
   }
   ```

## Versioning

Follow semantic versioning:

```bash
# Patch release (bug fixes)
npm version patch

# Minor release (new features)
npm version minor

# Major release (breaking changes)
npm version major
```

Then push the tag:
```bash
git push origin --tags
```
