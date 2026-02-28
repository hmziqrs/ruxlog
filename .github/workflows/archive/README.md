# Archived Workflows

This directory contains workflow files that have been retired from active use.

## release-builds.yml

**Archived Date:** 2025-01-15

**Reason:** The comprehensive release build workflow was failing consistently across all jobs. It was overly complex with too many build targets running in parallel, making it difficult to maintain and debug.

### What It Did

The `release-builds.yml` workflow was an ambitious multi-platform release system that built:

- **Desktop Linux**: x86_64 and ARM64, both webview and native renderers
- **Desktop Windows**: x86_64 and ARM64, both webview and native renderers  
- **Android**: x86_64 and ARM64, both webview and native renderers
- **Universal Archives**: Combined multi-architecture packages
- **GitHub Releases**: Automatic publishing of all artifacts

This resulted in a build matrix of 16+ parallel jobs that all needed to succeed.

### Replacement

The workflow has been replaced with `web-release.yml`, a slim GitHub Pages deployment that:
- Builds only web versions of the apps
- Deploys to GitHub Pages for easy access
- Has a much simpler build matrix (2 jobs instead of 16+)
- Is more reliable and faster to execute

### Future Use

This archived workflow may be useful when:
- Desktop and mobile builds are needed again
- Build infrastructure is more stable
- Platform-specific requirements are better understood
- A phased approach to multi-platform releases is ready

To restore it:
1. Move `release-builds.yml` back to `.github/workflows/`
2. Review and fix any issues that caused the original failures
3. Consider splitting into separate workflows per platform for better maintainability