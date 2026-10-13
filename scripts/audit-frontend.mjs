// Read-only inventory of frontend modules reachable from the application entrypoint.
// Run: node scripts/audit-frontend.mjs
import fs from "node:fs";
import path from "node:path";
import ts from "typescript";

const root = process.cwd();
const source = path.join(root, "src");
const extensions = [".ts", ".tsx", ".js", ".jsx", ".css", ".json"];
const files = [];
function walk(dir) {
  for (const item of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, item.name);
    if (item.isDirectory()) walk(full);
    else if (extensions.some((ext) => full.endsWith(ext))) files.push(full);
  }
}
walk(source);
const known = new Set(files);
function resolve(from, specifier) {
  if (!specifier.startsWith(".")) return null;
  const base = path.resolve(path.dirname(from), specifier);
  const choices = [base, ...extensions.map((ext) => base + ext), ...extensions.map((ext) => path.join(base, "index" + ext))];
  return choices.find((candidate) => known.has(candidate)) ?? null;
}
const edges = new Map();
const incoming = new Map(files.map((file) => [file, []]));
for (const file of files) {
  const deps = new Set();
  const content = fs.readFileSync(file, "utf8");
  if (/\.(?:ts|tsx|js|jsx)$/.test(file)) {
    const ast = ts.createSourceFile(file, content, ts.ScriptTarget.Latest, true, file.endsWith("tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
    const visit = (node) => {
      if ((ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) && node.moduleSpecifier && ts.isStringLiteral(node.moduleSpecifier)) {
        const dep = resolve(file, node.moduleSpecifier.text);
        if (dep) deps.add(dep);
      }
      if (ts.isCallExpression(node) && node.arguments.length && ts.isStringLiteral(node.arguments[0]) && (node.expression.kind === ts.SyntaxKind.ImportKeyword || node.expression.getText(ast) === "require")) {
        const dep = resolve(file, node.arguments[0].text);
        if (dep) deps.add(dep);
      }
      ts.forEachChild(node, visit);
    };
    visit(ast);
  } else if (file.endsWith(".css")) {
    for (const match of content.matchAll(/@import\s+(?:url\()?["']([^"']+)["']/g)) {
      const dep = resolve(file, match[1]);
      if (dep) deps.add(dep);
    }
  }
  edges.set(file, deps);
  for (const dep of deps) incoming.get(dep).push(file);
}
const entry = path.join(source, "main.tsx");
const reachable = new Set();
function visit(file) {
  if (!file || reachable.has(file)) return;
  reachable.add(file);
  for (const dep of edges.get(file) ?? []) visit(dep);
}
visit(entry);
const relative = (file) => path.relative(root, file).replaceAll(path.sep, "/");
const components = files.filter((file) => relative(file).startsWith("src/components/") && /\.tsx?$/.test(file));
const active = components.filter((file) => reachable.has(file));
const inactive = components.filter((file) => !reachable.has(file));
const lines = (file) => fs.readFileSync(file, "utf8").split("\n").length;
console.log("Frontend import graph (static imports/exports and literal dynamic imports)");
console.log(`Source files: ${files.length}; components: ${components.length}; reachable components: ${active.length}; unreachable components: ${inactive.length}`);
for (const [label, group] of [["REACHABLE", active], ["NOT REACHABLE FROM MAIN", inactive]]) {
  console.log(`\n${label} (${group.length}):`);
  for (const file of group.sort()) {
    const parents = (incoming.get(file) ?? []).map(relative);
    console.log(`  ${relative(file)} (${lines(file)} lines) <- ${parents.join(", ") || "no static imports"}`);
  }
}
console.log("\nUnreachable modules can still be imported by other unreachable modules, tests, or non-frontend entrypoints. Review before deleting.");
