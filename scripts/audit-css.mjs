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
