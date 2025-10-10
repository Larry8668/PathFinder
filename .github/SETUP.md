# GitHub Actions Setup Guide

This guide will help you set up automated Windows builds for PathFinder using GitHub Actions.

## 🚀 Quick Setup

### 1. Enable GitHub Actions
- Go to your repository on GitHub
- Click on the "Actions" tab
- Click "I understand my workflows, go ahead and enable them"

### 2. Optional: Code Signing (Recommended for Production)

If you want to sign your Windows executables for better security and user trust:

#### Generate Code Signing Certificate
```bash
# Install OpenSSL (if not already installed)
# Then generate a self-signed certificate for testing
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

#### Add Secrets to GitHub Repository
1. Go to your repository on GitHub
2. Click "Settings" → "Secrets and variables" → "Actions"
3. Click "New repository secret"
4. Add these secrets (optional for unsigned builds):
   - `TAURI_PRIVATE_KEY`: Your private key for code signing
   - `TAURI_KEY_PASSWORD`: Password for your private key

### 3. Test the Workflow

1. Push your changes to the `main` branch
2. Go to the "Actions" tab in your repository
3. You should see the "Build Windows App" workflow running
4. Wait for it to complete (usually 5-10 minutes)

## 📦 Build Artifacts

The workflow creates two types of Windows installers:

### MSI Installer
- **Location**: `src-tauri/target/release/bundle/msi/`
- **File**: `PathFinder_0.1.0_x64_en-US.msi`
- **Use**: Standard Windows installer, integrates with Windows Add/Remove Programs

### NSIS Installer
- **Location**: `src-tauri/target/release/bundle/nsis/`
- **File**: `PathFinder_0.1.0_x64-setup.exe`
- **Use**: Portable installer, smaller file size

## 🔧 Workflow Configuration

### Triggers
- **Push to main**: Builds and creates a draft release
- **Pull Request to main**: Builds for testing (no release)

### Build Process
1. **Checkout**: Downloads your code
2. **Setup Node.js**: Installs Node.js 18 with npm cache
3. **Setup Rust**: Installs Rust toolchain
4. **Install Dependencies**: Runs `npm ci` and `cargo fetch`
5. **Build Frontend**: Runs `npm run build`
6. **Build Tauri App**: Runs `npm run tauri:build`
7. **Upload Artifacts**: Saves installers as GitHub artifacts
8. **Create Release**: Creates a draft release (main branch only)

## 🐛 Troubleshooting

### Common Issues

**Build fails with "cargo not found":**
- The workflow should handle this automatically
- If it persists, check the Rust setup step

**Build fails with "npm not found":**
- Check Node.js version in workflow (currently 18)
- Ensure package.json has correct scripts

**Code signing fails:**
- Check that `TAURI_PRIVATE_KEY` and `TAURI_KEY_PASSWORD` secrets are set
- Verify the private key format is correct

**Artifacts not uploading:**
- Check file paths in the upload steps
- Ensure the build completed successfully

### Debug Steps

1. **Check workflow logs:**
   - Go to Actions tab
   - Click on the failed workflow run
   - Expand the failed step to see error details

2. **Test locally:**
   ```bash
   npm run tauri:build
   ```

3. **Check file paths:**
   - Verify the artifact paths match your build output
   - Check that the files exist after build

## 📋 Manual Release Process

If you prefer to create releases manually:

1. Go to "Releases" in your repository
2. Click "Create a new release"
3. Download artifacts from the latest workflow run
4. Upload the MSI and NSIS installers
5. Add release notes and publish

## 🔄 Customization

### Change Build Triggers
Edit `.github/workflows/build-windows.yml`:
```yaml
on:
  push:
    branches: [ main, develop ]  # Add more branches
  schedule:
    - cron: '0 0 * * 0'  # Weekly builds
```

### Add More Platforms
Create additional workflow files:
- `.github/workflows/build-macos.yml`
- `.github/workflows/build-linux.yml`

### Modify Build Commands
Update the workflow steps to customize the build process.

## 📊 Monitoring

- **Build Status**: Check the Actions tab for build status
- **Artifacts**: Download from the Actions tab or Releases
- **Notifications**: GitHub will email you on build failures (if enabled)

## 🎯 Next Steps

1. **Test the workflow** with a small change
2. **Set up code signing** for production releases
3. **Configure release automation** for automatic publishing
4. **Add more platforms** if needed
5. **Set up notifications** for build status

---

**Need help?** Check the [Tauri documentation](https://tauri.app/v1/guides/building/) or create an issue in this repository.
