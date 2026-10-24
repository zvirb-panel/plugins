// Fails on any `uses:` in .github/workflows that isn't pinned to a full commit
// SHA, because a moved tag runs new code with the job's token. Local actions
// (./path) and docker:// references with a digest are allowed.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const dir = ".github/workflows";
const pinned = /^[\w.-]+\/[\w.-]+(\/[\w./-]+)?@[0-9a-f]{40}$/;
const digest = /^docker:\/\/\S+@sha256:[0-9a-f]{64}$/;

const files = existsSync(dir) ? readdirSync(dir).filter((name) => /\.ya?ml$/.test(name)) : [];
if (files.length === 0) {
  console.log("no workflows to check");
  process.exit(0);
}

const refused = [];
for (const name of files) {
  readFileSync(join(dir, name), "utf8")
    .split(/\r?\n/)
    .forEach((line, index) => {
      const match = line.match(/^\s*(?:-\s+)?uses:\s*["']?([^\s"'#]+)/);
      if (!match) return;
      const ref = match[1];
      if (ref.startsWith("./") || pinned.test(ref) || digest.test(ref)) return;
      refused.push(`${dir}/${name}:${index + 1}: ${ref}`);
    });
}

if (refused.length > 0) {
  console.error("actions must be pinned to a 40-character commit SHA (owner/repo@<sha> # <tag>):");
  for (const line of refused) console.error(`  ${line}`);
  process.exit(1);
}
console.log(`${files.length} workflow file(s), every action pinned`);
