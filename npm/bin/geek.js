#!/usr/bin/env node
// Thin shim. After `npm install`, install.js drops the native binary next to
// this file as `geek` (Unix) or `geek.exe` (Windows). This shim execs it,
// passing through argv + stdio so output stays unbuffered for agents.
const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

const ext = process.platform === "win32" ? ".exe" : "";
const bin = path.join(__dirname, `geek${ext}`);
if (!fs.existsSync(bin)) {
  console.error(`[geek-cli] native binary missing at ${bin}`);
  console.error(`[geek-cli] re-run install: npm rebuild geek-cli`);
  process.exit(1);
}

const child = spawn(bin, process.argv.slice(2), { stdio: "inherit" });
child.on("exit", (code, signal) => {
  if (signal) process.kill(process.pid, signal);
  else process.exit(code ?? 0);
});
