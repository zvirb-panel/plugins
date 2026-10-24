// A command that can't start (not installed, not on PATH) leaves no exit status,
// so a check would fail without saying why. Call this after spawnSync to name the
// command and the error, and exit.
export function exitIfNotStarted(command, result) {
  if (!result.error) return;
  console.error(`could not start ${command}: ${result.error.message}`);
  process.exit(1);
}
