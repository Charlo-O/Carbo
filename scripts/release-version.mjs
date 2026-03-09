import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const rootDir = process.cwd();
const packageJsonPath = path.join(rootDir, 'package.json');
const tauriConfigPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');

const compareRefArgIndex = process.argv.indexOf('--compare-ref');
const compareRef =
  compareRefArgIndex >= 0 ? process.argv[compareRefArgIndex + 1] : process.env.COMPARE_REF;

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function readCargoVersion(content) {
  const packageSectionMatch = content.match(/\[package\][\s\S]*?(?=\n\[|$)/);
  const packageSection = packageSectionMatch ? packageSectionMatch[0] : content;
  const versionMatch = packageSection.match(/^\s*version\s*=\s*"([^"]+)"/m);

  if (!versionMatch) {
    throw new Error('Unable to read version from src-tauri/Cargo.toml');
  }

  return versionMatch[1];
}

function readCurrentVersions() {
  return {
    packageVersion: readJson(packageJsonPath).version,
    tauriVersion: readJson(tauriConfigPath).version,
    cargoVersion: readCargoVersion(fs.readFileSync(cargoTomlPath, 'utf8')),
  };
}

function versionsAreSynced(versions) {
  return versions.packageVersion === versions.tauriVersion && versions.packageVersion === versions.cargoVersion;
}

function readVersionAtRef(ref, filePath, type) {
  if (!ref || /^0+$/.test(ref)) {
    return null;
  }

  try {
    const normalizedPath = filePath.replace(/\\/g, '/');
    const fileContent = execFileSync('git', ['show', `${ref}:${normalizedPath}`], {
      cwd: rootDir,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    });

    if (type === 'json') {
      return JSON.parse(fileContent).version ?? null;
    }

    if (type === 'cargo') {
      return readCargoVersion(fileContent);
    }

    return null;
  } catch {
    return null;
  }
}

function writeOutput(name, value) {
  if (!process.env.GITHUB_OUTPUT) {
    console.log(`${name}=${value}`);
    return;
  }

  fs.appendFileSync(process.env.GITHUB_OUTPUT, `${name}=${value}\n`);
}

const currentVersions = readCurrentVersions();

if (!versionsAreSynced(currentVersions)) {
  console.error(
    [
      'Version mismatch detected.',
      `package.json: ${currentVersions.packageVersion}`,
      `src-tauri/tauri.conf.json: ${currentVersions.tauriVersion}`,
      `src-tauri/Cargo.toml: ${currentVersions.cargoVersion}`,
    ].join('\n'),
  );
  process.exit(1);
}

const previousPackageVersion = readVersionAtRef(compareRef, 'package.json', 'json');
const previousTauriVersion = readVersionAtRef(compareRef, 'src-tauri/tauri.conf.json', 'json');
const previousCargoVersion = readVersionAtRef(compareRef, 'src-tauri/Cargo.toml', 'cargo');

const previousVersion = previousPackageVersion ?? previousTauriVersion ?? previousCargoVersion ?? '';
const currentVersion = currentVersions.packageVersion;
const versionChanged = previousVersion !== '' && previousVersion !== currentVersion;

writeOutput('current_version', currentVersion);
writeOutput('previous_version', previousVersion);
writeOutput('version_changed', String(versionChanged));
