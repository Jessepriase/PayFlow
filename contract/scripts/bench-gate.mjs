#!/usr/bin/env node
// FlowPay instruction-budget gate.
//
// Runs the benchmarks from `src/bench.rs` through `cargo test --lib bench
// -- --nocapture`, compares the measured CPU instruction cost of every
// core entry-point against the budgets pinned in `bench-budgets.toml` and
// exits non-zero — naming the entry-point — on any violation.
//
//   node scripts/bench-gate.mjs                       # runs the bench itself
//   node scripts/bench-gate.mjs --log path/to/log     # gates a captured run
//
// Options:
//   --log <file>     gate an already captured `cargo test` log instead of
//                    running the benches (used by CI so the step is
//                    continue-on-error and failures are reported here).
//   --config <file>  budget config (default: bench-budgets.toml).
//
// Exit codes: 0 = every entry-point is within budget, 1 = gate failed.

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const contractDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const BENCH_ARGS = ["test", "--lib", "bench", "--", "--nocapture", "--test-threads=1"];

function parseArgs(argv) {
  const options = { log: null, config: join(contractDir, "bench-budgets.toml") };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--log") {
      options.log = resolve(argv[++i] ?? "");
    } else if (argv[i] === "--config") {
      options.config = resolve(argv[++i] ?? "");
    } else {
      throw new Error(`unknown argument: ${argv[i]}`);
    }
  }
  return options;
}

// Minimal TOML reader for the `[[budget]]` subset of bench-budgets.toml:
// comments, string values and integer values only. Keeping the parser in the
// gate avoids a parser dependency in a job that must stay fast and auditable.
function parseBudgetConfig(toml) {
  const budgets = [];
  const meta = {};
  let section = null;

  for (const rawLine of toml.split("\n")) {
    const line = rawLine.replace(/#.*$/, "").trim();
    if (line === "") continue;

    const sectionMatch = /^\[\[(\w+)\]\]$/.exec(line);
    if (sectionMatch) {
      section = sectionMatch[1] === "budget" ? {} : null;
      if (section) budgets.push(section);
      continue;
    }

    const metaMatch = /^\[(\w+)\]$/.exec(line);
    if (metaMatch) {
      section = section === null ? meta : null;
      continue;
    }

    const entry = /^(\w+)\s*=\s*(.+)$/.exec(line);
    if (!entry || !section) continue;
    const value = entry[2].trim();
    section[entry[1]] = /^".*"$/.test(value) ? value.slice(1, -1) : Number(value.replace(/_/g, ""));
  }

  if (budgets.length === 0) {
    throw new Error("no [[budget]] entries found in the budget config");
  }
  return { meta, budgets };
}

// `pub const MAX_X_INSTRUCTIONS: u64 = 4_620_000;`
function parseBenchConstants(source) {
  const constants = new Map();
  const re = /pub\s+const\s+(MAX_[A-Z0-9_]+)\s*:\s*u64\s*=\s*([0-9_]+)\s*;/g;
  let match;
  while ((match = re.exec(source)) !== null) {
    constants.set(match[1], Number(match[2].replace(/_/g, "")));
  }
  return constants;
}

// Each bench prints "\n[<bench>]\n  CPU Instructions : <n>\n  Memory Bytes : <n>".
function parseBenchOutput(log) {
  const measured = new Map();
  let current = null;

  for (const rawLine of log.split("\n")) {
    const line = rawLine.trim();

    const block = /^\[(\w+)\]$/.exec(line);
    if (block) {
      current = block[1];
      continue;
    }

    const cpu = /^CPU Instructions\s*:\s*([0-9]+)$/.exec(line);
    if (cpu && current) {
      measured.set(current, Number(cpu[1]));
      current = null;
    }
  }
  return measured;
}

function formatNumber(value) {
  return value.toLocaleString("en-US");
}

function main() {
  const options = parseArgs(process.argv.slice(2));

  const { meta, budgets } = parseBudgetConfig(readFileSync(options.config, "utf8"));
  const constants = parseBenchConstants(readFileSync(join(contractDir, "src", "bench.rs"), "utf8"));

  let log;
  if (options.log) {
    log = readFileSync(options.log, "utf8");
  } else {
    log = execFileSync("cargo", BENCH_ARGS, {
      cwd: contractDir,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
  }

  const compileFailure = log.split("\n").find((line) => /^error(?:\[[A-Z0-9]+\])?:/.test(line.trim()));
  if (compileFailure) {
    console.error("bench-gate: `cargo test` did not build — no budgets could be enforced.");
    console.error(`  ${compileFailure.trim()}`);
    process.exit(1);
  }

  const measured = parseBenchOutput(log);
  const failures = [];
  const rows = [];

  for (const budget of budgets) {
    const { entrypoint, bench, const: constName, max_cpu_instructions: max } = budget;
    const cpu = measured.get(bench);

    if (cpu === undefined) {
      failures.push(`${entrypoint}: bench "${bench}" produced no measurement (did it fail to run?)`);
      rows.push({ entrypoint, cpu: null, max, status: "MISSING" });
      continue;
    }

    const pinned = constants.get(constName);
    if (pinned !== max) {
      failures.push(
        `${entrypoint}: budget drift — bench-budgets.toml pins ${formatNumber(max)} but ` +
          `src/bench.rs declares ${constName} = ${pinned === undefined ? "missing" : formatNumber(pinned)}`,
      );
    }

    const status = cpu <= max ? "ok" : "OVER BUDGET";
    if (status !== "ok") {
      failures.push(
        `${entrypoint}: ${formatNumber(cpu)} CPU instructions exceeds the pinned budget of ` +
          `${formatNumber(max)} (baseline ${meta.headroom ?? "n/a"}, recorded ${meta.recorded_on ?? "n/a"})`,
      );
    }
    rows.push({ entrypoint, cpu, max, status });
  }

  console.log("");
  console.log("FlowPay instruction-budget gate");
  console.log(`  sdk ${meta.sdk ?? "n/a"} (soroban-env-host ${meta.soroban_env_host ?? "n/a"})`);
  console.log(`  budgets recorded ${meta.recorded_on ?? "n/a"} — ${meta.headroom ?? "n/a"}`);
  console.log("");
  for (const row of rows) {
    const cpu = row.cpu === null ? "n/a".padStart(13) : formatNumber(row.cpu).padStart(13);
    const headroom =
      row.cpu === null ? "" : `${(((row.max - row.cpu) / row.max) * 100).toFixed(1)}% headroom`;
    console.log(
      `  ${row.entrypoint.padEnd(22)} ${cpu} / ${formatNumber(row.max).padStart(13)}  ${row.status.padEnd(10)} ${headroom}`,
    );
  }
  console.log("");

  if (failures.length > 0) {
    console.error("bench-gate: instruction budgets violated");
    for (const failure of failures) {
      console.error(`  - ${failure}`);
    }
    console.error("");
    console.error("See contract/bench-budgets.toml for the headroom and re-pin policy.");
    process.exit(1);
  }

  console.log("bench-gate: all core entry-points within their pinned budgets.");
}

try {
  main();
} catch (error) {
  console.error(`bench-gate: ${error.message}`);
  process.exit(1);
}
