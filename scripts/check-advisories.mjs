// Scans every lockfile for known advisories. An accepted advisory is written in
// osv-scanner.toml with a reason and an ignoreUntil date, so it expires.
import { spawnSync } from "node:child_process";
import { exitIfNotStarted } from "./spawn.mjs";

const scan = spawnSync(
  "osv-scanner",
  ["scan", "source", "--recursive", "--allow-no-lockfiles", "--config", "osv-scanner.toml", "."],
  { stdio: "inherit" },
);
exitIfNotStarted("osv-scanner", scan);
process.exit(scan.status ?? 1);
