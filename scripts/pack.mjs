// Packs a built plugin as the archive Zvirb installs: a tar.gz of its
// zvirb-plugin.json, the files that names, its points/ folder and its module
// as plugin.wasm, all at the root. Prints what plugin-registry's add-version
// script reads: the plugin's index entry and the archive's url, sha256 and size.
// The url is the release tag's asset of the archive's file name, both decided
// by the caller, which uploads it there.
//   node scripts/pack.mjs <plugin id> <archive path> <release tag>
// Build the module first: cargo build --locked --target wasm32-unknown-unknown --release -p <plugin id>
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";

const [id, archivePath, tag] = process.argv.slice(2);
if (id === undefined || archivePath === undefined || tag === undefined) {
  console.error("usage: node scripts/pack.mjs <plugin id> <archive path> <release tag>");
  process.exit(1);
}

function fail(message) {
  console.error(`pack ${id}: ${message}`);
  process.exit(1);
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { encoding: "utf8", ...options });
  if (result.error !== undefined) fail(`could not start ${command}: ${result.error.message}`);
  if (result.status !== 0) fail(`${command} ${args.join(" ")} failed:\n${result.stderr}`);
  return result.stdout;
}

const metadata = JSON.parse(run("cargo", ["metadata", "--no-deps", "--format-version", "1"]));
const crate = metadata.packages.find((p) => p.name === id);
if (crate === undefined) fail("no crate of the workspace has that name");
if (!crate.description) fail("its Cargo.toml has no description, which the index shows as its summary");
const folder = dirname(crate.manifest_path);
const manifest = JSON.parse(readFileSync(join(folder, "zvirb-plugin.json"), "utf8"));
if (manifest.id !== id) fail(`zvirb-plugin.json names plugin ${manifest.id}`);
if (manifest.version !== crate.version) {
  fail(`zvirb-plugin.json says version ${manifest.version}, and Cargo.toml says ${crate.version}`);
}

// What the host reads: the files the manifest names, and the point files of
// the points it declares, which sit under points/.
const files = ["zvirb-plugin.json"];
for (const engine of manifest.engines ?? []) {
  if (engine.settings) files.push(engine.settings);
  files.push(...(engine.views ?? []));
}
if (existsSync(join(folder, "points"))) files.push("points");

const module = join(metadata.target_directory, "wasm32-unknown-unknown", "release", `${id.replaceAll("-", "_")}.wasm`);
if (!existsSync(module)) fail(`${module} isn't built`);

const staging = mkdtempSync(join(tmpdir(), `pack-${id}-`));
try {
  for (const file of files) cpSync(join(folder, file), join(staging, file), { recursive: true });
  cpSync(module, join(staging, "plugin.wasm"));
  // Sorted, with no owner or time, so packing the same build twice gives the
  // same bytes.
  const archive = resolve(archivePath);
  run("tar", [
    "--sort=name", "--owner=0", "--group=0", "--numeric-owner", "--mtime=@0",
    "--use-compress-program=gzip -n", "-cf", archive, "-C", staging, ".",
  ]);
  const bytes = readFileSync(archive);
  const release = {
    plugin: { id, name: manifest.name, summary: crate.description, games: manifest.games },
    version: crate.version,
    pdk: manifest.pdk,
    archive: {
      url: `https://github.com/zvirb-panel/plugins/releases/download/${tag}/${basename(archive)}`,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      size: statSync(archive).size,
    },
  };
  console.log(JSON.stringify(release, null, 2));
} finally {
  rmSync(staging, { recursive: true, force: true });
}
