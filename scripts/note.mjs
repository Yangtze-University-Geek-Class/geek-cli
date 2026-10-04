#!/usr/bin/env node
// 执行记录工具（docs/05）：add / index / check / selftest
// 仅用 Node 标准库（>=16）；固定 +08:00 时区；只追加、不改写既有记录。

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const STAGES = ["开工", "方案", "开发", "提交", "推送", "PR", "审查", "返工", "合并", "发布", "验收", "阻塞", "收尾"];
const DATE_RE = /^\d{4}-\d{2}-\d{2}$/;
const TIME_RE = /^\d{2}:\d{2}:\d{2}$/;
const ENTRY_RE = /^## (\d{2}:\d{2}:\d{2}) \+08:00 · ([^·]+?) · (.*)$/;

function fail(message) {
  console.error(`[note] ${message}`);
  process.exit(1);
}

function parseArgs(argv) {
  const args = { _: [] };
  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (token.startsWith("--")) {
      const key = token.slice(2);
      const next = argv[i + 1];
      if (next === undefined || next.startsWith("--")) args[key] = true;
      else {
        args[key] = next;
        i += 1;
      }
    } else args._.push(token);
  }
  return args;
}

function nowBeijing() {
  const shifted = new Date(Date.now() + 8 * 3600 * 1000);
  const pad = (n) => String(n).padStart(2, "0");
  return {
    date: `${shifted.getUTCFullYear()}-${pad(shifted.getUTCMonth() + 1)}-${pad(shifted.getUTCDate())}`,
    time: `${pad(shifted.getUTCHours())}:${pad(shifted.getUTCMinutes())}:${pad(shifted.getUTCSeconds())}`,
  };
}

function bumpOneSecond(stamp) {
  const [date, time] = stamp.split(" ");
  const [y, m, d] = date.split("-").map(Number);
  const [hh, mm, ss] = time.split(":").map(Number);
  const next = new Date(Date.UTC(y, m - 1, d, hh, mm, ss) + 1000);
  const pad = (n) => String(n).padStart(2, "0");
  return `${next.getUTCFullYear()}-${pad(next.getUTCMonth() + 1)}-${pad(next.getUTCDate())} ${pad(next.getUTCHours())}:${pad(next.getUTCMinutes())}:${pad(next.getUTCSeconds())}`;
}

function runGit(args) {
  try {
    return execFileSync("git", args, { encoding: "utf8" }).trim();
  } catch {
    return null;
  }
}

function readTree(root) {
  const chains = new Map();
  if (!existsSync(root)) return chains;
  for (const date of readdirSync(root)) {
    const dateDir = join(root, date);
    if (!DATE_RE.test(date) || !statSync(dateDir).isDirectory()) continue;
    for (const user of readdirSync(dateDir)) {
      const userDir = join(dateDir, user);
      if (!statSync(userDir).isDirectory()) continue;
      for (const file of readdirSync(userDir)) {
        if (!file.endsWith(".md")) continue;
        const path = join(userDir, file);
        const content = readFileSync(path, "utf8");
        const list = chains.get(file) ?? [];
        list.push({ date, user, path, content });
        chains.set(file, list);
      }
    }
  }
  for (const list of chains.values()) list.sort((a, b) => a.date.localeCompare(b.date));
  return chains;
}

function parseEntries(content) {
  const entries = [];
  const lines = content.split(/\r?\n/);
  for (let i = 0; i < lines.length; i += 1) {
    const match = ENTRY_RE.exec(lines[i].trim());
    if (!match) continue;
    const body = [];
    for (let j = i + 1; j < lines.length && !ENTRY_RE.test(lines[j].trim()); j += 1) body.push(lines[j]);
    entries.push({ time: match[1], stage: match[2].trim(), title: match[3].trim(), line: i + 1, body });
  }
  return entries;
}

function chainRecords(root, file) {
  return readTree(root).get(file) ?? [];
}

function lastStamp(records) {
  let last = null;
  for (const record of records) {
    for (const entry of parseEntries(record.content)) {
      const stamp = `${record.date} ${entry.time}`;
      if (last === null || stamp > last) last = stamp;
    }
  }
  return last;
}

function addEntry(root, options) {
  const stage = options.stage;
  const title = options.title;
  const by = options.by;
  const user = options.user;
  if (!STAGES.includes(stage)) fail(`阶段不在词表（docs/05 §5）：${stage}`);
  if (!title) fail("缺少标题（--title）");
  if (!by) fail("缺少执行者：--by 或 GEEK_NOTES_BY（docs/05 §5）");
  if (!options.did) fail("缺少「做了什么」（--did）");
  if (!options.result) fail("缺少「结果」（--result）");
  if (!user) fail("缺少负责人：--user / GEEK_NOTES_USER / git config user.name");

  const chainName = options.chain ?? runGit(["rev-parse", "--abbrev-ref", "HEAD"]);
  if (!chainName || chainName === "HEAD") fail("无法确定分支：--chain <分支名>");
  const file = `${chainName.toLowerCase().replace(/[/-]/g, "_")}.md`;
  if (!/^[a-z0-9_]+\.md$/.test(file)) fail(`链路文件名不合规（应为小写字母/数字/下划线）：${file}`);

  const now = nowBeijing();
  let date = options.date ?? now.date;
  let time = options.time ?? now.time;
  if (!DATE_RE.test(date) || !TIME_RE.test(time)) fail("--date/--time 格式应为 YYYY-MM-DD / HH:MM:SS");

  const records = chainRecords(root, file);
  const last = lastStamp(records);
  if (last !== null && `${date} ${time}` <= last) {
    const bumped = bumpOneSecond(last);
    date = bumped.slice(0, 10);
    time = bumped.slice(11);
    console.error(`[note] 为保证时间递增，已调整为 ${date} ${time}`);
  }

  const dir = join(root, date, user);
  mkdirSync(dir, { recursive: true });
  const path = join(dir, file);
  const isNew = !existsSync(path);
  const issueLabel = options.issue === undefined || options.issue === true ? "无 issue" : String(options.issue).startsWith("#") ? String(options.issue) : `#${options.issue}`;
  let text = "";
  if (isNew) text += `# ${chainName} · ${user} · ${date}\n\n负责人：${user}\n`;
  text += `\n## ${time} +08:00 · ${stage} · ${issueLabel} · ${title}\n\n`;
  text += `- 执行者：${by}\n- 做了什么：${options.did}\n- 结果：${options.result}\n`;
  if (options.next) text += `- 下一步：${options.next}\n`;
  const previous = isNew ? "" : readFileSync(path, "utf8");
  writeFileSync(path, previous + text, "utf8");
  return path;
}

function renderIndex(root) {
  const chains = readTree(root);
  const dates = [...new Set([...chains.values()].flat().map((record) => record.date))].sort().reverse();
  let out = `# 执行记录索引\n\n> 生成物：\`node scripts/note.mjs index\`（勿手工维护）；最后生成：${new Date().toISOString()}\n`;
  for (const date of dates) {
    out += `\n## ${date}\n`;
    const perUser = new Map();
    for (const [file, records] of chains) {
      for (const record of records) {
        if (record.date !== date) continue;
        const list = perUser.get(record.user) ?? [];
        list.push({ file, count: parseEntries(record.content).length });
        perUser.set(record.user, list);
      }
    }
    for (const [user, items] of [...perUser.entries()].sort((a, b) => a[0].localeCompare(b[0]))) {
      items.sort((a, b) => a.file.localeCompare(b.file));
      out += `- ${user}：${items.map((item) => `\`${item.file}\`（${item.count} 条）`).join("、")}\n`;
    }
  }
  return out;
}

function normalizeIndex(text) {
  return text.replace(/最后生成：.*/, "最后生成：<ts>");
}

function runCheck(root) {
  const problems = [];
  const chains = readTree(root);
  let entryCount = 0;
  for (const [file, records] of chains) {
    if (!/^[a-z0-9_]+\.md$/.test(file)) problems.push(`${file}: 文件名应为小写字母 / 数字 / 下划线`);
    let previousStamp = null;
    let firstStage = null;
    let seenTail = false;
    for (const record of records) {
      const lines = record.content.split(/\r?\n/);
      const titleLine = lines.find((line) => line.startsWith("# "));
      if (!titleLine || !titleLine.includes(` · ${record.user} · ${record.date}`)) {
        problems.push(`${record.path}: 标题行应与目录一致（# <分支> · ${record.user} · ${record.date}）`);
      }
      if (!lines.some((line) => line.trim() === `负责人：${record.user}`)) {
        problems.push(`${record.path}: 缺少或错误的「负责人：${record.user}」`);
      }
      if (lines.some((line) => /^(<{7}|={7}|>{7})(\s|$)/.test(line))) {
        problems.push(`${record.path}: 检测到冲突标记，记录文件被污染`);
      }
      for (const entry of parseEntries(record.content)) {
        entryCount += 1;
        if (!STAGES.includes(entry.stage)) problems.push(`${record.path}:${entry.line}: 阶段不在词表「${entry.stage}」`);
        if (firstStage === null) firstStage = entry.stage;
        if (seenTail) problems.push(`${record.path}:${entry.line}: 「收尾」之后不应再有记录`);
        if (entry.stage === "收尾") seenTail = true;
        for (const field of ["- 执行者：", "- 做了什么：", "- 结果："]) {
          if (!entry.body.some((line) => line.startsWith(field))) problems.push(`${record.path}:${entry.line}: 缺少必填字段「${field.slice(2, -1)}」`);
        }
        const stamp = `${record.date} ${entry.time}`;
        if (previousStamp !== null && stamp < previousStamp) problems.push(`${record.path}:${entry.line}: 时间倒退（${stamp} < ${previousStamp}）`);
        previousStamp = stamp;
      }
    }
    if (firstStage !== null && firstStage !== "开工") problems.push(`${file}: 链路第一条应为「开工」（实际「${firstStage}」）`);
  }

  const indexPath = join(root, "INDEX.md");
  if (chains.size > 0) {
    if (!existsSync(indexPath)) problems.push("INDEX.md: 缺失（运行 `node scripts/note.mjs index` 生成）");
    else if (normalizeIndex(readFileSync(indexPath, "utf8")) !== normalizeIndex(renderIndex(root))) {
      problems.push("INDEX.md: 已过期（重新运行 `node scripts/note.mjs index`）");
    }
  }
  return { problems, chains: chains.size, entries: entryCount };
}

function selfTest() {
  const dir = mkdtempSync(join(tmpdir(), "note-selftest-"));
  const root = join(dir, "notes");
  let ok = true;
  const report = (message) => {
    console.error(`selftest: ${message}`);
    ok = false;
  };
  try {
    const base = { by: "agent-selftest（selftest）", user: "tester", chain: "task/1/demo", issue: "#1" };
    addEntry(root, { ...base, stage: "开工", title: "创建链路", did: "selftest 建链路", result: "ok" });
    addEntry(root, { ...base, stage: "提交", title: "第一次提交", did: "selftest 提交", result: "ok" });

    const before = lastStamp(chainRecords(root, "task_1_demo.md"));
    addEntry(root, { ...base, stage: "推送", title: "时间倒填", did: "填一个更早的时间", result: "应被自动递增", date: "2000-01-01", time: "00:00:00" });
    const after = lastStamp(chainRecords(root, "task_1_demo.md"));
    if (!(after > before)) report("时间递增（倒填应被 +1s 修正）失败");

    writeFileSync(join(root, "INDEX.md"), renderIndex(root), "utf8");
    let result = runCheck(root);
    if (result.problems.length) report(`正确树未通过：\n  ${result.problems.join("\n  ")}`);

    const date = nowBeijing().date;
    const chainPath = join(root, date, "tester", "task_1_demo.md");
    writeFileSync(chainPath, readFileSync(chainPath, "utf8") + "\n## 23:59:59 +08:00 · 乱写阶段 · #1 · bad\n\n- 执行者：x\n- 做了什么：y\n- 结果：z\n", "utf8");
    result = runCheck(root);
    if (!result.problems.some((p) => p.includes("阶段不在词表"))) report("未检出坏阶段");
    if (!result.problems.some((p) => p.includes("INDEX.md"))) report("未检出 INDEX 过期");

    writeFileSync(chainPath, readFileSync(chainPath, "utf8") + "\n<<<<<<< HEAD\n", "utf8");
    result = runCheck(root);
    if (!result.problems.some((p) => p.includes("冲突标记"))) report("未检出冲突标记");

    console.log(ok ? "selftest: PASS" : "selftest: FAIL");
    process.exit(ok ? 0 : 1);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function main() {
  const [sub, ...rest] = process.argv.slice(2);
  const args = parseArgs(rest);
  const root = args.root === undefined || args.root === true ? "notes" : args.root;
  const clean = (value) => (value === true ? undefined : value);
  if (sub === "add") {
    const path = addEntry(root, {
      stage: args.stage,
      title: args.title,
      issue: clean(args.issue),
      did: args.did,
      result: args.result,
      next: clean(args.next),
      by: clean(args.by) ?? process.env.GEEK_NOTES_BY,
      user: clean(args.user) ?? process.env.GEEK_NOTES_USER ?? runGit(["config", "user.name"]),
      chain: clean(args.chain),
      date: clean(args.date),
      time: clean(args.time),
    });
    console.log(`已追加：${path}`);
  } else if (sub === "index") {
    mkdirSync(root, { recursive: true });
    writeFileSync(join(root, "INDEX.md"), renderIndex(root), "utf8");
    console.log(`已生成：${join(root, "INDEX.md")}`);
  } else if (sub === "check") {
    const { problems, chains, entries } = runCheck(root);
    if (problems.length) {
      for (const problem of problems) console.error(`[note] ${problem}`);
      process.exit(1);
    }
    console.log(`OK：${chains} 条链路 / ${entries} 条记录`);
  } else if (sub === "selftest") {
    selfTest();
  } else {
    console.error("用法：node scripts/note.mjs <add|index|check|selftest> [--root notes] [--stage ... --title ... --did ... --result ... --by ... --user ... --issue ...]");
    process.exit(1);
  }
}

main();
