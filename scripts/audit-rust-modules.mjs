#!/usr/bin/env node
// Conservative Rust module-reference inventory. This is NOT a compiler reachability graph.
// It deliberately counts textual references (including tests/comments) to avoid false certainty.
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const root = join(process.cwd(), 'src-tauri', 'src');
const files = readdirSync(root).filter((name) => name.endsWith('.rs')).sort();
const sources = new Map(files.map((name) => [name, readFileSync(join(root, name), 'utf8')]));
const entry = sources.get('lib.rs') ?? '';
const declared = new Set([...entry.matchAll(/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;/gm)].map((match) => match[1]));
const escaped = (value) => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const rows = files.filter((name) => !['lib.rs', 'main.rs'].includes(name)).map((name) => {
  const stem = name.slice(0, -3);
  const pattern = new RegExp('\\b' + escaped(stem) + '\\b');
  const references = [...sources].filter(([other, text]) => other !== name && pattern.test(text)).map(([other]) => other);
  return { module: stem, declared: declared.has(stem), references };
});
for (const row of rows) {
  console.log(`${row.declared ? 'DECLARED' : 'UNDECLARED'} ${row.module}: ${row.references.length ? row.references.join(', ') : '(no textual references in sibling Rust files)'}`);
}
console.log('\nCaution: textual inventory only. Inspect imports, include!/#[path], feature flags, build scripts, tests, and runtime safety hooks before deletion.');
