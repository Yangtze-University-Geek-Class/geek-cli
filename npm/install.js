#!/usr/bin/env node
// Downloads the prebuilt `geek` binary from the matching GitHub release
// based on platform + arch. Falls back to env GEEK_BINARY_URL if set.
const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const pkg = require("./package.json");
const REPO = "Yangtze-University-Geek-Class/geek-cli";
const TAG = process.env.GEEK_VERSION || `v${pkg.version}`;

const TARGETS = {
  "darwin-arm64":  "aarch64-apple-darwin",
  "darwin-x64":    "x86_64-apple-darwin",
  "linux-arm64":   "aarch64-unknown-linux-gnu",
  "linux-x64":     "x86_64-unknown-linux-gnu",
  "linux-x64-musl":"x86_64-unknown-linux-musl",
  "win32-x64":     "x86_64-pc-windows-msvc",
};

function detectTarget() {
  const p = process.platform;
  const a = process.arch === "x64" ? "x64" : process.arch === "arm64" ? "arm64" : null;
  if (!a) throw new Error(`unsupported arch: ${process.arch}`);
  const key = `${p}-${a}`;
  if (!TARGETS[key]) throw new Error(`unsupported platform: ${key}`);
  return TARGETS[key];
}

function get(url, redirects = 5) {
  return new Promise((resolve, reject) => {
    https.get(url, { headers: { "User-Agent": "geek-cli-install" } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location && redirects > 0) {
        return resolve(get(res.headers.location, redirects - 1));
      }
      if (res.statusCode !== 200) return reject(new Error(`download failed: HTTP ${res.statusCode}`));
      const chunks = [];
      res.on("data", (c) => chunks.push(c));
      res.on("end", () => resolve(Buffer.concat(chunks)));
      res.on("error", reject);
    }).on("error", reject);
  });
}

async function verify(buf, sha256Url) {
  try {
    const want = (await get(sha256Url)).toString().split(/\s+/)[0].trim();
    const crypto = require("crypto");
    const got = crypto.createHash("sha256").update(buf).digest("hex");
    if (want && got !== want) throw new Error(`sha256 mismatch (want ${want}, got ${got})`);
  } catch (e) {
    console.warn(`[geek-cli] sha256 verify skipped: ${e.message}`);
  }
}

async function main() {
  const target = detectTarget();
  const ext = process.platform === "win32" ? ".exe" : "";
  const binaryName = `geek-${target}${ext}`;
  const baseUrl = process.env.GEEK_BINARY_URL
    || `https://github.com/${REPO}/releases/download/${TAG}/${binaryName}`;
  const binDir = path.join(__dirname, "bin");
  fs.mkdirSync(binDir, { recursive: true });
  const dest = path.join(binDir, `geek${ext}`);

  console.log(`[geek-cli] downloading ${binaryName} (${TAG}) ...`);
  const buf = await get(baseUrl);
  await verify(buf, `${baseUrl}.sha256`);
  fs.writeFileSync(dest, buf);
  if (process.platform !== "win32") fs.chmodSync(dest, 0o755);

  // sanity check
  try {
    const v = execSync(`"${dest}" --version`, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
    console.log(`[geek-cli] installed: ${v.trim()}`);
  } catch {
    console.warn(`[geek-cli] binary written to ${dest} but --version check failed; run \`geek --version\` to debug`);
  }
}

main().catch((e) => {
  console.error(`[geek-cli] install failed: ${e.message}`);
  console.error(`[geek-cli] you can also install via cargo: cargo install --git https://github.com/${REPO}`);
  process.exit(0); // don't break npm install entirely
});
