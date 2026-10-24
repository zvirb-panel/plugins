// A tag can be moved after the fact, so every action is pinned to a commit.
// Allowed: owner/repo[/path]@<40 hex>, ./local actions and docker://…@sha256:….
import { existsSync, readdirSync, readFileSync } from "node:fs";

const dir = ".github/workflows";
const pinned = /^[\w.-]+\/[\w.-]+(\/[\w./-]+)?@[0-9a-f]{40}$/;
const image = /^docker:\/\/\S+@sha256:[0-9a-f]{64}$/;

const files = existsSync(dir)
  ? readdirSync(dir).filter((name) => name.endsWith(".yml"))
  : [];
if (files.length === 0) {
  console.log("no workflows to check");
  process.exit(0);
}

let failed = false;
for (const name of files) {
  const lines = readFileSync(`${dir}/${name}`, "utf8").split("\n");
  lines.forEach((line, index) => {
    const match = line.match(/^\s*(?:-\s+)?uses:\s*(\S+)/);
    if (!match) return;
    const ref = match[1].replace(/^["']|["']$/g, "");
    if (ref.startsWith("./") || pinned.test(ref) || image.test(ref)) return;
    console.error(`${dir}/${name}:${index + 1}: ${ref} is not pinned to a commit SHA`);
    failed = true;
  });
}

if (failed) process.exit(1);
console.log(`${files.length} workflow files, every action pinned`);
