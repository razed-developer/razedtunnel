// Read-only conservative Rust module reference graph.
// Run: node scripts/audit-rust-dependency-graph.mjs
import fs from "node:fs";
import path from "node:path";

const base = path.join(process.cwd(), "src-tauri/src");
const names = fs.readdirSync(base).filter((x) => x.endsWith(".rs")).sort();
const sources = new Map(names.map((x) => [x.slice(0, -3), fs.readFileSync(path.join(base, x), "utf8")]));
const modules = [...sources.keys()].filter((x) => x !== "lib" && x !== "main");
const lib = sources.get("lib") ?? "";
const declared = new Set([...lib.matchAll(/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;/gm)].map((x) => x[1]));
const edges = [];
for (const [from, text] of sources) {
  for (const to of modules) {
    if (from === to) continue;
    // Qualified calls and imports only; other Rust syntax is not fully parsed.
    const qualified = new RegExp("\\b" + to + "::");
    const directImport = new RegExp("\\buse\\s+crate::" + to + "\\b");
    const groupedImport = new RegExp("\\buse\\s+crate::\\{[^}]*\\b" + to + "\\b", "s");
    if (qualified.test(text) || directImport.test(text) || groupedImport.test(text)) edges.push({ from, to });
  }
}
const libBody = lib.replace(/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*;.*$/gm, "");
const roots = modules.filter((m) => new RegExp("\\b" + m + "::|\\buse\\s+crate::" + m + "\\b").test(libBody));
const visited = new Set(roots), queue = [...roots];
while (queue.length) {
  const current = queue.shift();
  for (const edge of edges) {
    if (edge.from !== current || visited.has(edge.to)) continue;
    visited.add(edge.to);
    queue.push(edge.to);
  }
}
const rows = modules.map((name) => ({ name, declared: declared.has(name), reachedByHeuristic: visited.has(name), incoming: edges.filter((e) => e.to === name).map((e) => e.from) }));
const report = {
  disclaimer: "Heuristic textual module-reference graph, NOT a compiler call graph. Unvisited modules may still be active via macros, trait dispatch, grouped imports, feature flags, startup hooks, generated code, tests or other entry points. Do not delete automatically.",
  counts: { declaredModules: declared.size, directLibRoots: roots.length, visitedByHeuristic: visited.size, reviewCandidates: rows.filter((r) => r.declared && !r.reachedByHeuristic).length },
  roots, edges, modules: rows,
};
const graph = ["flowchart LR", '  ROOT["lib.rs direct references"]'];
const ids = new Map(modules.map((m, i) => [m, "R" + i]));
for (const r of rows) {
  const id = ids.get(r.name);
  graph.push('  ' + id + '["' + r.name + '"]');
  if (roots.includes(r.name)) graph.push("  ROOT --> " + id);
  if (!r.reachedByHeuristic) graph.push("  class " + id + " review");
}
for (const e of edges) if (ids.has(e.from) && ids.has(e.to)) graph.push("  " + ids.get(e.from) + " --> " + ids.get(e.to));
graph.push("  classDef review fill:#fff0d9,stroke:#a76c19,color:#333");
const out = path.join(process.cwd(), "docs/audits");
fs.mkdirSync(out, { recursive: true });
fs.writeFileSync(path.join(out, "rust-module-graph.json"), JSON.stringify(report, null, 2) + "\n");
fs.writeFileSync(path.join(out, "rust-module-graph.mmd"), graph.join("\n") + "\n");
console.log(JSON.stringify(report.counts, null, 2));
console.log("Review candidates (not proven orphaned):");
for (const r of rows.filter((r) => r.declared && !r.reachedByHeuristic)) console.log("  " + r.name + " <- " + (r.incoming.join(", ") || "no qualified references"));
console.log("Wrote docs/audits/rust-module-graph.json and rust-module-graph.mmd");
