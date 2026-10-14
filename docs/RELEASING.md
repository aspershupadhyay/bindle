# Releasing Bindle

1. Update the version in `Cargo.toml` (`[workspace.package]`), `app/package.json` and `app/src-tauri/tauri.conf.json`, and move the changelog's "Unreleased" notes under the new version.
2. Make sure CI is green on `main` and every check in [ACCEPTANCE.md](ACCEPTANCE.md) passes.
3. Tag and push (`git tag v0.1.0 && git push origin v0.1.0`), or start the **Release** workflow by hand from the Actions tab and type the version; GitHub then makes the tag.
4. The **Release** workflow builds installers on Windows (`.msi`, `.exe`), macOS (Apple silicon and Intel `.dmg`) and Linux (`.deb`, `.rpm`, `.AppImage`) and puts them on a **draft** release.
5. Install each one on a real machine, run the hands-on checks, then publish the draft.

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
