#!/usr/bin/env node
// Read-only inventory of legacy Home AI references across current source files.
// Textual matches are evidence for review, not proof of runtime reachability.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

const roots = ['src', 'src-tauri/src'];
const ignored = new Set(['node_modules', 'dist', 'target', '.git']);
const names = /\b(?:HomeConversation(?:Message|Summary)?|HomeChat(?:StartResult|StreamEvent)|HomeProjectContextRequest|HomeContext(?:TextInput|ErrorInput|Source)|Model(?:HubSnapshot|TrialSnapshot|TrialResultView|Selection)|model_hub|model_trial|project_context|conversation::|legacy_background_services)\b/g;
const rows = [];
function walk(directory) {
  for (const entry of readdirSync(directory)) {
    if (ignored.has(entry)) continue;
    const path = join(directory, entry);
    if (statSync(path).isDirectory()) { walk(path); continue; }
    if (!/\.(?:rs|ts|tsx)$/.test(path)) continue;
    const lines = readFileSync(path, 'utf8').split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
      const matches = [...lines[i].matchAll(names)].map((match) => match[0]);
      if (matches.length) rows.push({ path: relative('.', path), line: i + 1, matches: [...new Set(matches)] });
    }
  }
}
for (const root of roots) walk(root);
for (const row of rows) console.log(`${row.path}:${row.line}: ${row.matches.join(', ')}`);
console.log(`\n${rows.length} matching source lines. Review imports, feature flags, test references and updater guards before changing behavior.`);
