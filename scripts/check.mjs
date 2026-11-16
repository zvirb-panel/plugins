// An empty Cargo workspace makes cargo commands fail or do nothing, so the
// no-members case is decided here instead of in mise.toml.
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { delimiter, resolve } from "node:path";

const TOOLS_ROOT = resolve("target/tools");

const run = (command, args, env = process.env) =>
  spawnSync(command, args, { stdio: "inherit", env }).status ?? 1;

const independent = [
  ["node", "scripts/check-actions.mjs"],
  ["node", "scripts/check-advisories.mjs"],
];

for (const [command, ...args] of independent) {
  if (run("node", ["scripts/header.mjs", command, ...args]) !== 0) process.exit(1);
  const status = run(command, args);
  if (status !== 0) process.exit(status);
}

const metadata = spawnSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
  encoding: "utf8",
});
if (metadata.status !== 0) {
  process.stderr.write(metadata.stderr);
  process.exit(metadata.status ?? 1);
}

if (JSON.parse(metadata.stdout).workspace_members.length === 0) {
  console.log("No plugins yet");
  process.exit(0);
}

// The plugins' tests validate each manifest against the schema `zvirb-pdk schema`
// prints, so the command must be the one from the PDK commit Cargo.lock locks.
const pdkSource = readFileSync("Cargo.lock", "utf8").match(
  /name = "zvirb-pdk"\nversion = "[^"]+"\nsource = "git\+([^?"]+)\?[^#"]*#([0-9a-f]+)"/,
);
if (!pdkSource) {
  console.error("Cargo.lock has no zvirb-pdk git source to install zvirb-pdk from");
  process.exit(1);
}
const [, pdkRepository, pdkRevision] = pdkSource;

const steps = [
  ["cargo", "fmt", "--check"],
  ["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"],
  [
    "cargo", "install", "--locked", "--git", pdkRepository, "--rev", pdkRevision,
    "--root", TOOLS_ROOT, "zvirb-pdk-cli",
  ],
  ["cargo", "test", "--locked"],
  ["cargo", "build", "--locked", "--target", "wasm32-unknown-unknown", "--release"],
];

const withTools = {
  ...process.env,
  PATH: `${TOOLS_ROOT}/bin${delimiter}${process.env.PATH}`,
};

for (const [command, ...args] of steps) {
  if (run("node", ["scripts/header.mjs", command, ...args]) !== 0) process.exit(1);
  const status = run(command, args, withTools);
  if (status !== 0) process.exit(status);
}
