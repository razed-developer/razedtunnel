// Read-only command exposure audit. Run: node scripts/audit-command-graph.mjs
// Writes Mermaid + JSON to docs/audits; does not delete or change production code.
// Static evidence only: Rust macro expansion, callbacks and dynamic dispatch are not resolved.
import fs from "node:fs";
import path from "node:path";
import ts from "typescript";

const root = process.cwd();
const read = (p) => fs.readFileSync(path.join(root, p), "utf8");
const rust = read("src-tauri/src/lib.rs");
const server = read("src-tauri/src/mcp_server.rs");
const sourceDir = path.join(root, "src");
const files = [];
function walk(dir) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p);
    else if (/\.[jt]sx?$/.test(p)) files.push(p);
  }
}
walk(sourceDir);
const calls = new Map();
const unresolved = [];
for (const file of files) {
  const code = fs.readFileSync(file, "utf8");
  const ast = ts.createSourceFile(file, code, ts.ScriptTarget.Latest, true, file.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
  const location = path.relative(root, file).replaceAll("\\", "/");
  function visit(node) {
    if (ts.isCallExpression(node) && /^(?:invoke|window\.__TAURI__\.core\.invoke)$/.test(node.expression.getText(ast))) {
      const arg = node.arguments[0];
      if (arg && (ts.isStringLiteral(arg) || ts.isNoSubstitutionTemplateLiteral(arg))) {
        const rows = calls.get(arg.text) ?? [];
        rows.push({ file: location, line: ast.getLineAndCharacterOfPosition(node.getStart(ast)).line + 1 });
        calls.set(arg.text, rows);
      } else unresolved.push({ file: location, expression: node.getText(ast).slice(0, 140) });
    }
    ts.forEachChild(node, visit);
  }
  visit(ast);
}
const handler = rust.match(/tauri::generate_handler!\s*\[([\s\S]*?)\]/);
if (!handler) throw Error("Cannot locate Tauri generate_handler macro; refusing incomplete report.");
const registered = [...handler[1].matchAll(/\b([A-Za-z_]\w*)::([A-Za-z_]\w*)\s*,/g)].map((m) => ({ module: m[1], command: m[2] }));
const rustSources = fs.readdirSync(path.join(root, "src-tauri/src")).filter((f) => f.endsWith(".rs"));
const rustContent = new Map(rustSources.map((f) => [f, read("src-tauri/src/" + f)]));
const tauri = registered.map((r) => {
  const file = r.module + ".rs";
  const source = rustContent.get(file) ?? "";
  const definition = new RegExp("#\\[tauri::command(?:\\([^)]*\\))?\\][\\s\\S]{0,300}?\\b(?:pub(?:\\([^)]*\\))?\\s+)?(?:async\\s+)?fn\\s+" + r.command + "\\b").test(source);
  return { ...r, definitionFound: definition, frontendCalls: calls.get(r.command) ?? [], status: calls.has(r.command) ? "frontend-referenced" : "no-static-frontend-call" };
});
const registeredNames = new Set(registered.map((x) => x.command));
const unregisteredCalls = [...calls].filter(([name]) => !registeredNames.has(name)).map(([command, sites]) => ({ command, sites }));
// MCP macro registrations: #[tool(...)] preceding method declarations, plus #[tool_router] impls.
// Tool methods are externally callable even without Rust call sites.
const mcp = [];
const lines = server.split(/\r?\n/);
let inTool = false;
for (let i = 0; i < lines.length; i++) {
  if (/^\s*#\[tool\s*(?:\(|\])/.test(lines[i])) inTool = true;
  if (inTool && /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)/.test(lines[i])) {
    const name = lines[i].match(/\bfn\s+(\w+)/)?.[1];
    if (name) mcp.push({ name, line: i + 1, exposure: "macro-tool-candidate" });
    inTool = false;
  }
}
// Keep graph compact: aggregate frontend sites into one node; show each command and each MCP tool.
const esc = (s) => s.replaceAll("\\", "\\\\").replaceAll('"', "'").replaceAll("\n", " ");
const diagram = ["flowchart LR", '  UI["Frontend invoke sites"]', '  IPC["Tauri IPC registry"]', '  MCP["MCP tool router"]', '  EXT["External MCP clients"]', "  EXT --> MCP", "  UI --> IPC"];
for (const [i, r] of tauri.entries()) {
  const id = "T" + i;
  diagram.push('  ' + id + '["' + esc(r.command) + '"]');
  diagram.push("  IPC --> " + id);
  if (!r.frontendCalls.length) diagram.push("  class " + id + " suspect");
}
for (const [i, r] of mcp.entries()) {
  const id = "M" + i;
  diagram.push('  ' + id + '["' + esc(r.name) + '"]');
  diagram.push("  MCP --> " + id);
}
diagram.push("  classDef suspect fill:#fff0d9,stroke:#a76c19,color:#333");
const report = {
  disclaimer: "Command-level static exposure inventory, NOT a Rust function reachability proof. MCP tool methods are external entry points. Dynamic invoke calls, aliases, feature flags, macros and startup hooks need manual review.",
  generatedAt: new Date().toISOString(),
  totals: { registeredTauri: tauri.length, frontendReferenced: tauri.filter((r) => r.frontendCalls.length).length, noStaticFrontendCall: tauri.filter((r) => !r.frontendCalls.length).length, mcpToolCandidates: mcp.length, dynamicFrontendCalls: unresolved.length },
  tauri, mcp, unregisteredCalls, unresolved,
};
const out = path.join(root, "docs/audits");
fs.mkdirSync(out, { recursive: true });
fs.writeFileSync(path.join(out, "command-graph.json"), JSON.stringify(report, null, 2) + "\n");
fs.writeFileSync(path.join(out, "command-graph.mmd"), diagram.join("\n") + "\n");
console.log(JSON.stringify(report.totals, null, 2));
console.log("Tauri commands with no literal frontend invoke (REVIEW, not automatically orphaned):");
for (const r of tauri.filter((x) => !x.frontendCalls.length)) console.log("  " + r.module + "::" + r.command);
if (unregisteredCalls.length) console.log("Unregistered literal frontend calls:", unregisteredCalls);
console.log("Wrote docs/audits/command-graph.json and command-graph.mmd");
