// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Prepares a release build: `node scripts/release-config.mjs v0.0.2`.
//
// 1. Writes the version into Cargo.toml, app/package.json and
//    app/src-tauri/tauri.conf.json, so a release needs no version commit.
// 2. When TAURI_UPDATER_PUBKEY is set, puts it in the updater settings and
//    turns on the signed update files (latest.json and its signatures) that
//    installed copies of Bindle download. The build then also needs
//    TAURI_SIGNING_PRIVATE_KEY. See docs/RELEASING.md.
//
// Run from app/. The release workflow runs it on a fresh checkout.

import { readFileSync, writeFileSync } from "node:fs";

const tag = process.argv[2] ?? "";
const version = tag.replace(/^v/, "");
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`"${tag}" is not a version such as v0.0.2`);
  process.exit(1);
}

const editJson = (path, edit) => {
  const data = JSON.parse(readFileSync(path, "utf8"));
  edit(data);
  writeFileSync(path, `${JSON.stringify(data, null, 2)}\n`);
};

editJson("package.json", (pkg) => {
  pkg.version = version;
});

const pubkey = (process.env.TAURI_UPDATER_PUBKEY ?? "").trim();
editJson("src-tauri/tauri.conf.json", (conf) => {
  conf.version = version;
  if (pubkey) {
    conf.plugins.updater.pubkey = pubkey;
    conf.bundle.createUpdaterArtifacts = true;
  }
});

const cargo = readFileSync("../Cargo.toml", "utf8");
const bumped = cargo.replace(/(\[workspace\.package\][^[]*?\nversion = )"[^"]*"/, `$1"${version}"`);
if (bumped === cargo && !cargo.includes(`version = "${version}"`)) {
  console.error("Could not find the version in Cargo.toml's [workspace.package]");
  process.exit(1);
}
writeFileSync("../Cargo.toml", bumped);

console.log(`Version ${version}. Update files: ${pubkey ? "on" : "off (no TAURI_UPDATER_PUBKEY)"}.`);
