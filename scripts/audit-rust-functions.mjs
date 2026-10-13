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
// Entry-point classifications are deliberately conservative: they suppress review,
// not establish proof of reachability. Attributes may precede declarations.
const lib = sources.get("lib.rs") ?? "";
const handler = lib.match(/tauri::generate_handler!\s*\[([\s\S]*?)\]/)?.[1] ?? "";
const registeredCommands = new Set([...handler.matchAll(/::([A-Za-z_]\w*)\b/g)].map((m) => m[1]));
const externallyExposed = (fn) => {
  const code = sources.get(fn.file) ?? "";
  const lines = code.split("\n");
  const preceding = lines.slice(Math.max(0, fn.line - 18), fn.line - 1);
  // Find attributes immediately before a function, not arbitrary nearby attributes.
  let i = preceding.length - 1;
  while (i >= 0 && !preceding[i].trim()) i--;
  const attributes = [];
  let depth = 0;
  while (i >= 0) {
    const line = preceding[i].trim();
    if (line.startsWith("#[") || depth > 0 || line.endsWith("]")) {
      attributes.unshift(line);
      depth += (line.match(/\]/g) ?? []).length - (line.match(/\[/g) ?? []).length;
      i--;
      if (depth > 0) continue;
    } else break;
  }
  const attrs = attributes.join(" ");
  const isMcpTool = fn.file.startsWith("mcp_") && /#\[tool(?:\s*\(|\s*\])/.test(attrs);
  const hasTauriAttribute = /#\[tauri::command\]/.test(attrs);
  const isTauri = registeredCommands.has(fn.name) && hasTauriAttribute;
  const isUnregisteredTauri = hasTauriAttribute && !registeredCommands.has(fn.name);
  const isTest = /#\[(?:tokio::)?test(?:\s*\(|\s*\])/.test(attrs);
  // ServerHandler trait hooks are invoked by MCP runtime dispatch.
  const isMcpServerHandler = fn.file === "mcp_server.rs" && ["on_initialized", "get_info", "list_resources", "read_resource"].includes(fn.name);
  // Known Deref trait implementation: Rust invokes this through trait dispatch.\n  const isTraitImpl = fn.file === "browser.rs" && fn.name === "deref" && /impl Deref for BrowserScope\\s*\\{/.test(code);\n  return { isMcpTool, isTauri, isUnregisteredTauri, isTest, isMcpServerHandler, isTraitImpl };
};
const rows = declarations.map((fn) => {
  const pattern = new RegExp("\\b" + fn.name + "\\b", "g");
  const references = [];
  for (const [file, code] of sources) {
    let count = [...code.matchAll(pattern)].length;
    if (file === fn.file) count -= 1;
    if (count > 0) references.push({ file, count });
  }
  const flags = externallyExposed(fn);
  return { ...fn, references, ...flags, reviewCandidate: references.length === 0 && !Object.values(flags).some(Boolean) };
});
const report = {
  disclaimer: "Heuristic textual references, NOT compiler reachability. Macro tools, callbacks, trait methods, attributes, feature flags, tests and string literals require manual review. Do not automatically delete.",
  counts: { declarations: rows.length, noTextualReferences: rows.filter((x) => x.references.length === 0).length, reviewCandidates: rows.filter((x) => x.reviewCandidate).length, protectedEntryPoints: rows.filter((x) => x.references.length === 0 && !x.reviewCandidate).length },
  functions: rows,
};
const out = path.join(process.cwd(), "docs/audits");
fs.mkdirSync(out, { recursive: true });
fs.writeFileSync(path.join(out, "rust-function-references.json"), JSON.stringify(report, null, 2) + "\n");
console.log(JSON.stringify(report.counts, null, 2));
console.log("Review candidates (manual inspection required):");
for (const row of rows.filter((x) => x.reviewCandidate)) {
  console.log("  " + row.file + ":" + row.line + " " + row.name);
}
console.log("Protected zero-reference functions by flag:");
for (const flag of ["isMcpTool", "isTauri", "isUnregisteredTauri", "isTest", "isMcpServerHandler"]) {
  const protectedRows = rows.filter((x) => x.references.length === 0 && x[flag]);
  console.log("  " + flag + ": " + protectedRows.length);
  for (const row of protectedRows.slice(0, 8)) console.log("    " + row.file + ":" + row.line + " " + row.name);
}
console.log("History settings symbols:");
for (const name of ["load_history_settings", "save_history_settings", "history_settings_path"]) {
  for (const row of rows.filter((x) => x.name === name)) console.log("  " + row.file + ":" + row.line + " " + name + " <- " + (row.references.map((x) => x.file + " (" + x.count + ")").join(", ") || "no other textual references"));
}
console.log("Wrote docs/audits/rust-function-references.json; no code removed.");
