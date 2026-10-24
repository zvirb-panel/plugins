// osv-scanner exits 128 when it finds no lockfile. A workspace that has none yet
// has nothing to scan, which is not a failure.
import { spawnSync } from "node:child_process";

const NO_PACKAGE_SOURCES = 128;

const result = spawnSync(
  "osv-scanner",
  ["scan", "source", "--recursive", "--config", "osv-scanner.toml", "."],
  { stdio: "inherit" },
);
if (result.error) {
  console.error(`could not start osv-scanner: ${result.error.message}`);
  process.exit(1);
}
if (result.status === NO_PACKAGE_SOURCES) {
  console.log("no lockfiles to scan");
  process.exit(0);
}
process.exit(result.status ?? 1);
