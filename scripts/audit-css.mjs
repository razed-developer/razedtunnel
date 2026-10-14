// Read-only CSS baseline for incremental stylesheet consolidation.
// Run: node scripts/audit-css.mjs
import { readFileSync } from "node:fs";
const files = ["src/styles.css", "src/workspace.css", "src/theme.css"];
const tracked = [".workspace-card", ".gateway-overview", ".public-overview", ".connection-state", ".desktop-shell", ".sidebar-nav-item"];
const totals = { lines: 0, bytes: 0, important: 0 };
for (const file of files) {
  const css = readFileSync(file, "utf8");
  const lines = css.split(/\r?\n/).length;
  const bytes = Buffer.byteLength(css);
  const important = (css.match(/!important\b/g) ?? []).length;
  totals.lines += lines;
  totals.bytes += bytes;
  totals.important += important;
  console.log(`\n${file}: ${lines} lines, ${(bytes / 1024).toFixed(1)} KiB, ${important} !important declarations`);
  for (const selector of tracked) {
    // Count literal selector mentions, including media queries and contextual selectors.
    const count = css.split(selector).length - 1;
    if (count) console.log(`  ${selector}: ${count} references`);
  }
}
console.log(`\nTotal: ${totals.lines} lines, ${(totals.bytes / 1024).toFixed(1)} KiB, ${totals.important} !important declarations`);
console.log("Counts are diagnostic, not a claim that every reference is a duplicate.");


// Identify large legacy feature blocks for follow-up review. This does not delete CSS.
const workspaceLines = readFileSync("src/workspace.css", "utf8").split(/\r?\n/);
const sections = [
  ["Model Hub", "/* Stage 2 Model Hub"],
  ["Home Chat", "/* Stage 3 · Local Home Chat"],
  ["Video Intelligence", "/* Video Intelligence is intentionally isolated"],
  ["Video Projects", "/* Video Projects file workspace"],
  ["HTTPS Setup", "/* HTTPS Setup: one guided"],
  ["Operational Home", "/* Operational home dashboard"],
];
console.log("\nLegacy section boundaries (review before removal):");
for (let i = 0; i < sections.length; i++) {
  const [name, marker] = sections[i];
  const start = workspaceLines.findIndex((line) => line.includes(marker));
  if (start < 0) { console.log(`  ${name}: marker missing`); continue; }
  const next = sections.slice(i + 1).map(([, m]) => workspaceLines.findIndex((line) => line.includes(m))).find((line) => line > start);
  console.log(`  ${name}: starts at line ${start + 1}${next == null ? " (no following boundary)" : `, next marker at ${next + 1}`}`);
}
console.log("Boundaries are landmarks, not proof that the intervening rules are unused.");
