// Read-only Rust function reference inventory. Run: node scripts/audit-rust-functions.mjs
// Textual evidence only; macros, traits, dynamic dispatch and comments can mislead.
import fs from "node:fs";
import path from "node:path";
const base = path.join(process.cwd(), "src-tauri/src");
const files = fs.readdirSync(base).filter((f) => f.endsWith(".rs")).sort();
const sources = new Map(files.map((f) => [f, fs.readFileSync(path.join(base, f), "utf8")]));
const declarations = [];
for (const [file, code] of sources) {
  for (const match of code.matchAll(/^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_]\w*)\s*(?:<[^;\n]*>)?\s*\(/gm)) {
    declarations.push({ file, line: code.slice(0, match.index).split("\n").length, name: match[1] });
  }
}
const rows = declarations.map((fn) => {
  const pattern = new RegExp("\\b" + fn.name + "\\b", "g");
  const references = [];
  for (const [file, code] of sources) {
    let count = [...code.matchAll(pattern)].length;
    if (file === fn.file) count -= 1;
    if (count > 0) references.push({ file, count });
  }
  return { ...fn, references, reviewCandidate: references.length === 0 };
});
const report = {
  disclaimer: "Heuristic textual references, NOT compiler reachability. Macro tools, callbacks, trait methods, attributes, feature flags, tests and string literals require manual review. Do not automatically delete.",
  counts: { declarations: rows.length, noTextualReferences: rows.filter((x) => x.reviewCandidate).length },
  functions: rows,
};
const out = path.join(process.cwd(), "docs/audits");
fs.mkdirSync(out, { recursive: true });
fs.writeFileSync(path.join(out, "rust-function-references.json"), JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify(report.counts, null, 2));
console.log("History settings symbols:");
for (const name of ["load_history_settings", "save_history_settings", "history_settings_path"]) {
  for (const row of rows.filter((x) => x.name === name)) console.log("  " + row.file + ":" + row.line + " " + name + " <- " + (row.references.map((x) => x.file + " (" + x.count + ")").join(", ") || "no other textual references"));
}
console.log("Wrote docs/audits/rust-function-references.json; no code removed.");
