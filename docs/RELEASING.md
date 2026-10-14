# Releasing Bindle

1. Move the changelog's "Unreleased" notes under the new version.
2. Make sure CI is green on `main` and every check in [ACCEPTANCE.md](ACCEPTANCE.md) passes.
3. Start the **Release** workflow from the Actions tab and type the version (such as `v0.0.2`), or tag and push (`git tag v0.0.2 && git push origin v0.0.2`). The version is taken from the tag and written into the app during the build, so nothing needs bumping by hand. It must be higher than the last release, or installed copies will not offer it.
4. The **Release** workflow builds installers on Windows (`.msi`, `.exe`), macOS (Apple silicon and Intel `.dmg`) and Linux (`.deb`, `.rpm`, `.AppImage`) and publishes them on a release, with `latest.json` for updates.
5. Within six hours, every installed copy of Bindle (v0.0.2 or later) shows "Bindle 0.0.x is here" and installs it when the person agrees.

## Updates inside the app

Bindle checks `https://github.com/aspershupadhyay/bindle/releases/latest/download/latest.json` shortly after it opens and every six hours. Each update file is signed with the updater key, and Bindle refuses any file whose signature does not match the public key built into it. This key is separate from the Apple and Windows certificates, is free, and works without them.

**One-time setup (owner only).** Until this is done, releases build without update files and installed copies hear about nothing.

1. On your own computer, in `app/`, run `npx tauri signer generate -w ~/.tauri/bindle-updater.key` and choose a password. It writes the private key and `bindle-updater.key.pub`.
2. In the GitHub repository, under **Settings → Secrets and variables → Actions**, add three secrets:

| Secret | What it is |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | The whole contents of `bindle-updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | The password you chose |
| `TAURI_UPDATER_PUBKEY` | The whole contents of `bindle-updater.key.pub` |

3. Keep the private key and its password backed up somewhere safe, such as a password manager. If either is lost, installed copies can no longer be updated and everyone has to install the next version by hand.

Releases made before this setup (v0.0.1) cannot update themselves. People on them install the next version once from the download page; after that, updates arrive by themselves.

## Signing

Without these secrets the workflow still builds, but the installers are unsigned. Windows SmartScreen and macOS Gatekeeper will then warn people before they open Bindle. Add them under **Settings → Secrets and variables → Actions** in the GitHub repository.

| Secret | What it is |
|---|---|
| `APPLE_CERTIFICATE` | The "Developer ID Application" certificate exported as `.p12`, then base64 encoded |
| `APPLE_CERTIFICATE_PASSWORD` | The password chosen when exporting the `.p12` |
| `APPLE_SIGNING_IDENTITY` | For example `Developer ID Application: Sparsh Upadhyay (TEAMID)` |
| `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` | For notarisation: the Apple ID, an app-specific password, and the team ID |
| `WINDOWS_CERTIFICATE` | An Authenticode code-signing certificate as `.pfx`, base64 encoded |
| `WINDOWS_CERTIFICATE_PASSWORD` | Its password |

Linux packages are not signed. The release lists SHA-256 checksums for them.

Never commit a certificate, key or password. They live only in repository secrets.
