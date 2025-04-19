# Developer Guide: CI/CD & Release Process for rustored

This document describes how the GitHub Actions workflows are structured for `rustored` and how to trigger builds, tests, Docker image pushes, and new binary releases.

---

## GitHub Actions Workflows

### 1. Continuous Integration (CI)
- **Workflow:** `.github/workflows/ci.yml`
- **Triggers:** On every push or pull request to `main`.
- **What it does:**
  - Installs Rust toolchain
  - Caches dependencies
  - Runs `cargo test`

### 2. Docker Image Build & Push
- **Workflow:** `.github/workflows/docker.yml`
- **Triggers:** On every push to `main`.
- **What it does:**
  - Builds a multi-stage, distroless Docker image for `rustored`
  - Pushes the image to GitHub Container Registry (`ghcr.io/<owner>/rustored:latest` and with the commit SHA tag)

### 3. Release Binaries & Multi-Platform Docker Images
- **Workflow:** `.github/workflows/release.yml`
- **Triggers:** On every push of a semver tag (`v*.*.*`, e.g., `v1.0.0`)
- **What it does:**
  - Builds and tests on Linux, macOS, and Windows
  - Uploads built binaries as release assets
  - Builds and uploads Docker images for all platforms
  - Creates a GitHub Release with the binaries and Docker images attached

---

## How to Build and Release the Docker Image

### Automated (via GitHub Actions)
- **On every push to `main`,** the workflow `.github/workflows/docker.yml` will:
  - Build the Docker image using the multi-stage, distroless `Dockerfile`.
  - Push the image to GitHub Container Registry:
    - `ghcr.io/<owner>/rustored:latest`
    - `ghcr.io/<owner>/rustored:<commit-sha>`
- **On every semver tag (e.g., `v1.2.3`),** the release workflow will also push:
    - `ghcr.io/<owner>/rustored:v1.2.3`

### Manual (local build & push)
If you want to build and push the Docker image manually:

```sh
# Build the image locally
DOCKER_BUILDKIT=1 docker build -t ghcr.io/<owner>/rustored:dev .

# Log in to GitHub Container Registry
echo $GITHUB_TOKEN | docker login ghcr.io -u <your-username> --password-stdin

# Push the image
docker push ghcr.io/<owner>/rustored:dev
```

Replace `<owner>` and `<your-username>` with your GitHub org/user. `$GITHUB_TOKEN` should be a Personal Access Token with `write:packages` scope for manual pushes.

---

## How to Trigger a New Release

1. **Update your code and commit all changes to `main`.**

2. **Create and push a new semver tag:**
   ```sh
   git tag v1.2.3  # Replace with your new version
   git push origin v1.2.3
   ```
   This will:
   - Build and test on all platforms
   - Create a GitHub Release at https://github.com/<owner>/rustored/releases/tag/v1.2.3
   - Attach Linux, macOS, and Windows binaries
   - Attach Docker images to the release

3. **Check the release status:**
   - The release workflow will appear in the Actions tab.
   - If successful, the release will be published with all assets attached.

---

## Notes
- The Docker image for each commit to `main` is always available at:
  - `ghcr.io/<owner>/rustored:latest`
  - `ghcr.io/<owner>/rustored:<commit-sha>`
- The Docker image for a tagged release is available at:
  - `ghcr.io/<owner>/rustored:<version>`
- All workflow files can be found in `.github/workflows/`.

---

For questions or to update the CI/CD process, edit the relevant workflow YAML files in `.github/workflows/` and update this documentation as needed.
