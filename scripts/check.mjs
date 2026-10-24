// An empty Cargo workspace makes cargo commands fail or do nothing, so the
// no-members case is decided here instead of in mise.toml.
import { spawnSync } from "node:child_process";

const run = (command, args) =>
  spawnSync(command, args, { stdio: "inherit" }).status ?? 1;

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

const steps = [
  ["cargo", "fmt", "--check"],
  ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
  ["cargo", "test"],
  ["cargo", "build", "--target", "wasm32-unknown-unknown", "--release"],
];

for (const [command, ...args] of steps) {
  if (run("node", ["scripts/header.mjs", command, ...args]) !== 0) process.exit(1);
  const status = run(command, args);
  if (status !== 0) process.exit(status);
}
