/**
 * Start core FX services + Tauri (or Vite) without parallel `cargo run` (Windows locks
 * target/debug/*.exe). One `cargo build` then run binaries directly.
 *
 * Starts (in order):
 *   matching (8083) → risk (8084) → liquidity (8091) →
 *   execution-engine (8092, in-process Rust AI scorer) → gateway → UI
 *
 * Optional remote Python AI: set AI_EXECUTION_MODE=http and DEV_STACK_WITH_AI=1
 * (starts ai/ai-execution-service on 8093). Default needs no venv.
 */
import { execSync, spawn } from 'node:child_process';
import fs from 'node:fs';
import net from 'node:net';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const uiRoot = path.join(__dirname, '..');
const repoRoot = path.join(uiRoot, '..', '..');
const aiRoot = path.join(repoRoot, 'ai', 'ai-execution-service');
const isWin = process.platform === 'win32';

const stackTargetDir = process.env.DEV_STACK_TARGET_DIR
  ? path.resolve(process.env.DEV_STACK_TARGET_DIR)
  : path.join(repoRoot, 'target', 'dev-stack');
const stackDebugDir = path.join(stackTargetDir, 'debug');

function exeName(bin) {
  return path.join(stackDebugDir, isWin ? `${bin}.exe` : bin);
}

const matchExe = exeName('matching-engine-service');
const riskExe = exeName('risk-service');
const liqExe = exeName('liquidity-graph-service');
const execExe = exeName('execution-engine');
const gwExe = exeName('gateway-service');
const smcExe = exeName('fx-smc-advisory-api');

/** @type {import('node:child_process').ChildProcess[]} */
const children = [];

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

function killStaleServices() {
  if (process.env.DEV_STACK_NO_KILL === '1') {
    console.log('[dev-stack] DEV_STACK_NO_KILL=1 — not stopping existing stack processes.');
    return;
  }
  console.log(
    '[dev-stack] Stopping any previous matching / risk / liquidity / execution / gateway / AI processes…',
  );
  if (isWin) {
    for (const im of [
      'matching-engine-service.exe',
      'risk-service.exe',
      'liquidity-graph-service.exe',
      'execution-engine.exe',
      'gateway-service.exe',
      'fx-smc-advisory-api.exe',
    ]) {
      try {
        execSync(`taskkill /IM ${im} /F`, { stdio: 'ignore' });
      } catch {
        /* not running */
      }
    }
    // Free AI port 8093 / SMC 8094 if prior processes hold them
    for (const port of [8093, 8094]) {
      try {
        execSync(
          `powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort ${port} -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }"`,
          { stdio: 'ignore' },
        );
      } catch {
        /* ignore */
      }
    }
  } else {
    for (const name of [
      'matching-engine-service',
      'risk-service',
      'liquidity-graph-service',
      'execution-engine',
      'gateway-service',
      'fx-smc-advisory-api',
    ]) {
      try {
        execSync(`pkill -x ${name}`, { stdio: 'ignore' });
      } catch {
        /* ignore */
      }
    }
    try {
      execSync('fuser -k 8093/tcp', { stdio: 'ignore' });
    } catch {
      /* ignore */
    }
    try {
      execSync('fuser -k 8094/tcp', { stdio: 'ignore' });
    } catch {
      /* ignore */
    }
  }
}

async function waitHttpHealth(url, predicate, timeoutMs = 45000) {
  const deadline = Date.now() + timeoutMs;
  let lastErr = '';
  while (Date.now() < deadline) {
    try {
      const r = await fetch(url);
      const text = await r.text();
      if (!r.ok) {
        lastErr = `HTTP ${r.status}`;
      } else if (predicate(text)) {
        return;
      } else {
        const clip = text.replace(/\s+/g, ' ').slice(0, 200);
        lastErr = clip ? `unexpected body: ${clip}` : 'empty body';
      }
    } catch (e) {
      lastErr = e instanceof Error ? e.message : String(e);
    }
    await sleep(250);
  }
  throw new Error(`Timeout waiting for ${url} (${lastErr})`);
}

function canBindPort(port, host = '0.0.0.0') {
  return new Promise((resolve) => {
    const s = net.createServer();
    s.once('error', () => resolve(false));
    s.listen(port, host, () => {
      s.close(() => resolve(true));
    });
  });
}

const SKIP_AUTO_GATEWAY_PORTS = new Set([
  8081, 8082, 8083, 8084, 8085, 8086, 8091, 8092, 8093, 8094, 50051,
]);

function jsonHealthMatchesService(text, expectedService) {
  try {
    const j = JSON.parse(text);
    if (typeof j !== 'object' || j == null) return false;
    if (String(j.status ?? '').toLowerCase() !== 'healthy') return false;
    return j.service === expectedService;
  } catch {
    return false;
  }
}

function aiHealthOk(text) {
  try {
    const j = JSON.parse(text);
    const status = String(j.status ?? '').toLowerCase();
    return status === 'ok' || status === 'healthy';
  } catch {
    return false;
  }
}

async function resolveGatewayPort() {
  const explicit = process.env.GATEWAY_HTTP_PORT;
  if (explicit !== undefined && explicit !== '') {
    const p = parseInt(explicit, 10);
    if (Number.isNaN(p) || p < 1 || p > 65535) {
      throw new Error(`Invalid GATEWAY_HTTP_PORT=${explicit}`);
    }
    if (!(await canBindPort(p))) {
      throw new Error(
        `GATEWAY_HTTP_PORT=${p} is in use. Stop the other process or unset GATEWAY_HTTP_PORT to auto-pick a free port.`,
      );
    }
    return p;
  }
  for (let p = 8080; p <= 8099; p++) {
    if (SKIP_AUTO_GATEWAY_PORTS.has(p)) continue;
    if (await canBindPort(p)) {
      if (p !== 8080) {
        console.log(
          `[dev-stack] Port 8080 is busy; using gateway port ${p} (VITE_API_URL=http://127.0.0.1:${p} for the UI).`,
        );
      }
      return p;
    }
  }
  throw new Error(
    'No free TCP port for gateway in range 8080–8099 (after skipping compose defaults).',
  );
}

function killAll() {
  for (const c of children) {
    if (!c?.pid || c.exitCode != null) continue;
    if (isWin) {
      try {
        execSync(`taskkill /PID ${c.pid} /T /F`, { stdio: 'ignore' });
      } catch {
        try {
          c.kill();
        } catch {
          /* ignore */
        }
      }
    } else {
      try {
        c.kill('SIGTERM');
      } catch {
        /* ignore */
      }
    }
  }
}

/**
 * @param {string} label
 * @param {string} binary
 * @param {string} healthUrl
 * @param {(t: string) => boolean} healthPred
 * @param {NodeJS.ProcessEnv} [env]
 * @param {string[]} [args]
 */
async function startRustService(label, binary, healthUrl, healthPred, env = process.env, args = []) {
  console.log(`[dev-stack] Starting ${label}…`);
  const child = spawn(binary, args, {
    cwd: repoRoot,
    stdio: 'inherit',
    env,
  });
  children.push(child);
  child.on('exit', (code) => {
    if (code !== 0 && code !== null) {
      console.error(`[dev-stack] ${label} exited with code`, code);
      killAll();
      process.exit(code);
    }
  });
  try {
    await waitHttpHealth(healthUrl, healthPred);
    console.log(`[dev-stack] ${label} is up (HTTP /health OK).`);
  } catch (e) {
    console.error('[dev-stack]', e instanceof Error ? e.message : e);
    killAll();
    process.exit(1);
  }
}

function resolvePython() {
  const venvPy = isWin
    ? path.join(aiRoot, '.venv', 'Scripts', 'python.exe')
    : path.join(aiRoot, '.venv', 'bin', 'python');
  if (fs.existsSync(venvPy)) return venvPy;
  return isWin ? 'py' : 'python3';
}

async function startAiExecutionService() {
  if (process.env.DEV_STACK_WITH_AI !== '1') {
    console.log(
      '[dev-stack] Using in-process Rust AI scorer (default). Set DEV_STACK_WITH_AI=1 + AI_EXECUTION_MODE=http for remote Python.',
    );
    return;
  }

  const py = resolvePython();
  const args = py === 'py' ? ['-3', 'main.py'] : ['main.py'];
  console.log(`[dev-stack] Starting remote AI execution service (8093) with ${py}…`);

  const child = spawn(py, args, {
    cwd: aiRoot,
    stdio: 'inherit',
    env: {
      ...process.env,
      PORT: '8093',
      HOST: '127.0.0.1',
    },
    shell: isWin && py === 'py',
  });
  children.push(child);
  child.on('exit', (code) => {
    if (code !== 0 && code !== null) {
      console.warn(
        `[dev-stack] AI execution service exited with code ${code} (engine will fall back if in HTTP mode).`,
      );
    }
  });

  try {
    await waitHttpHealth('http://127.0.0.1:8093/health', aiHealthOk, 60000);
    console.log('[dev-stack] AI execution service is up (HTTP /health OK).');
  } catch (e) {
    console.warn(
      '[dev-stack] AI execution service did not become ready:',
      e instanceof Error ? e.message : e,
    );
    console.warn(
      '[dev-stack] Create a venv first: cd ai/ai-execution-service && python -m venv .venv && .venv\\Scripts\\pip install -r requirements.txt',
    );
  }
}

async function main() {
  process.on('SIGINT', () => {
    killAll();
    process.exit(0);
  });
  process.on('SIGTERM', () => {
    killAll();
    process.exit(0);
  });

  killStaleServices();
  await sleep(750);

  console.log(
    '[dev-stack] Building into',
    stackTargetDir,
    '(CARGO_TARGET_DIR — avoids overwriting a running target/debug/*.exe on Windows)…',
  );
  const cargoEnv = { ...process.env, CARGO_TARGET_DIR: stackTargetDir };
  execSync(
    'cargo build -p matching-engine-service -p risk-service -p liquidity-graph-service -p execution-engine -p gateway-service -p fx-smc-advisory-api',
    {
      cwd: repoRoot,
      stdio: 'inherit',
      shell: isWin,
      env: cargoEnv,
    },
  );

  await startRustService(
    'matching-engine-service (8083)',
    matchExe,
    'http://127.0.0.1:8083/health',
    (t) => jsonHealthMatchesService(t, 'matching-engine-service'),
  );

  await startRustService(
    'risk-service (8084)',
    riskExe,
    'http://127.0.0.1:8084/health',
    (t) => jsonHealthMatchesService(t, 'risk-service'),
  );

  await startRustService(
    'liquidity-graph-service (8091)',
    liqExe,
    'http://127.0.0.1:8091/health',
    (t) => jsonHealthMatchesService(t, 'liquidity-graph-service'),
  );

  // AI before execution-engine only when using remote HTTP mode
  await startAiExecutionService();

  const execEnv = { ...process.env };
  if (process.env.DEV_STACK_WITH_AI === '1') {
    execEnv.AI_EXECUTION_MODE = process.env.AI_EXECUTION_MODE || 'http';
    execEnv.AI_EXECUTION_URL = process.env.AI_EXECUTION_URL || 'http://127.0.0.1:8093';
  } else {
    execEnv.AI_EXECUTION_MODE = process.env.AI_EXECUTION_MODE || 'local';
  }

  await startRustService(
    'execution-engine (8092)',
    execExe,
    'http://127.0.0.1:8092/health',
    (t) => jsonHealthMatchesService(t, 'execution-engine'),
    execEnv,
  );

  let gwPort;
  try {
    gwPort = await resolveGatewayPort();
  } catch (e) {
    console.error('[dev-stack]', e instanceof Error ? e.message : e);
    killAll();
    process.exit(1);
  }

  const stackEnv = {
    ...process.env,
    GATEWAY_HTTP_PORT: String(gwPort),
    VITE_API_URL: `http://127.0.0.1:${gwPort}`,
  };

  await startRustService(
    `gateway-service (port ${gwPort})`,
    gwExe,
    `http://127.0.0.1:${gwPort}/health`,
    (t) => jsonHealthMatchesService(t, 'gateway'),
    stackEnv,
  );

  await startRustService(
    'fx-smc-advisory-api (8094)',
    smcExe,
    'http://127.0.0.1:8094/health',
    (t) => jsonHealthMatchesService(t, 'fx-smc-advisory-api'),
    { ...stackEnv, SMC_API_PORT: '8094', VITE_SMC_API_URL: 'http://127.0.0.1:8094' },
    ['config/default.toml'],
  );

  const webOnly = process.env.DEV_STACK_WEB_ONLY === '1';
  const uiScript = webOnly ? 'dev' : 'tauri:dev';
  console.log(
    webOnly
      ? '[dev-stack] Starting Vite web UI (DEV_STACK_WEB_ONLY=1)…'
      : '[dev-stack] Starting Tauri desktop UI…',
  );
  const ui = spawn('npm', ['run', uiScript], {
    cwd: uiRoot,
    stdio: 'inherit',
    shell: isWin,
    env: {
      ...stackEnv,
      VITE_SMC_API_URL: 'http://127.0.0.1:8094',
    },
  });
  children.push(ui);

  ui.on('exit', (code) => {
    killAll();
    process.exit(code ?? 0);
  });
}

main().catch((e) => {
  console.error('[dev-stack]', e);
  killAll();
  process.exit(1);
});
