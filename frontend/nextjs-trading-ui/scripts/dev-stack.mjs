/**
 * Start matching engine → gateway → Next.js without parallel `cargo run` (Windows locks
 * target/debug/*.exe and the package cache). One `cargo build` then run binaries directly.
 *
 * Stops prior Rust dev binaries by default (avoids Windows 10048): matching, liquidity graph,
 * execution engine, gateway.
 * Set DEV_STACK_NO_KILL=1 to skip. Readiness uses HTTP /health, not raw TCP connect.
 * If GATEWAY_HTTP_PORT is unset and 8080 is busy, picks another port in 8080–8099, skipping
 * host ports used by deploy/docker-compose.yml (8081–8086, 8091, 50051) so we do not collide
 * with market-data, pricing, matching, etc. Also starts liquidity-graph (8091) and execution-engine (8092)
 * so gateway proxies `/liquidity/*` and `/execution/*` work locally.
 */
import { execSync, spawn } from 'node:child_process';
import net from 'node:net';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const uiRoot = path.join(__dirname, '..');
const repoRoot = path.join(uiRoot, '..', '..');
const isWin = process.platform === 'win32';

/**
 * Separate target dir so `cargo build` never has to replace `target/debug/*.exe` while an old
 * instance is still running (Windows returns Access denied / os error 5).
 * Override with DEV_STACK_TARGET_DIR.
 */
const stackTargetDir = process.env.DEV_STACK_TARGET_DIR
  ? path.resolve(process.env.DEV_STACK_TARGET_DIR)
  : path.join(repoRoot, 'target', 'dev-stack');
const stackDebugDir = path.join(stackTargetDir, 'debug');

const matchExe = path.join(
  stackDebugDir,
  isWin ? 'matching-engine-service.exe' : 'matching-engine-service',
);
const liqExe = path.join(
  stackDebugDir,
  isWin ? 'liquidity-graph-service.exe' : 'liquidity-graph-service',
);
const execExe = path.join(stackDebugDir, isWin ? 'execution-engine.exe' : 'execution-engine');
const gwExe = path.join(stackDebugDir, isWin ? 'gateway-service.exe' : 'gateway-service');

/** @type {import('node:child_process').ChildProcess[]} */
const children = [];

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

/** Free 8083 / 50051 / 8080 from a prior dev run so the new binaries can bind (Windows 10048). */
function killStaleRustServices() {
  if (process.env.DEV_STACK_NO_KILL === '1') {
    console.log('[dev-stack] DEV_STACK_NO_KILL=1 — not stopping existing matching/gateway processes.');
    return;
  }
  console.log(
    '[dev-stack] Stopping any previous matching / liquidity-graph / execution-engine / gateway binaries…',
  );
  if (isWin) {
    for (const im of [
      'matching-engine-service.exe',
      'liquidity-graph-service.exe',
      'execution-engine.exe',
      'gateway-service.exe',
    ]) {
      try {
        execSync(`taskkill /IM ${im} /F`, { stdio: 'ignore' });
      } catch {
        /* not running */
      }
    }
  } else {
    for (const name of [
      'matching-engine-service',
      'liquidity-graph-service',
      'execution-engine',
      'gateway-service',
    ]) {
      try {
        execSync(`pkill -x ${name}`, { stdio: 'ignore' });
      } catch {
        /* ignore */
      }
    }
  }
}

/**
 * Wait until GET url returns 200 and predicate(body) is true (confirms *this* process, not a stale listener).
 */
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

/** True if nothing is listening on host:port (same bind family as gateway: 0.0.0.0). */
function canBindPort(port, host = '0.0.0.0') {
  return new Promise((resolve) => {
    const s = net.createServer();
    s.once('error', () => resolve(false));
    s.listen(port, host, () => {
      s.close(() => resolve(true));
    });
  });
}

/** Host ports other microservices use in deploy/docker-compose.yml (avoid when auto-picking gateway). */
const SKIP_AUTO_GATEWAY_PORTS = new Set([
  8081, 8082, 8083, 8084, 8085, 8086, 8091, 8092, 50051,
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

/**
 * Port for gateway HTTP. If GATEWAY_HTTP_PORT is set, that port must be free. Otherwise scan 8080–8099.
 */
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
          `[dev-stack] Port 8080 is busy; using gateway port ${p} (NEXT_PUBLIC_API_URL=http://127.0.0.1:${p} for Next.js).`,
        );
      }
      return p;
    }
  }
  throw new Error('No free TCP port for gateway in range 8080–8099 (after skipping compose defaults).');
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

async function main() {
  process.on('SIGINT', () => {
    killAll();
    process.exit(0);
  });
  process.on('SIGTERM', () => {
    killAll();
    process.exit(0);
  });

  killStaleRustServices();
  await sleep(750);

  console.log(
    '[dev-stack] Building into',
    stackTargetDir,
    '(CARGO_TARGET_DIR — avoids overwriting a running target/debug/*.exe on Windows)…',
  );
  const cargoEnv = { ...process.env, CARGO_TARGET_DIR: stackTargetDir };
  execSync(
    'cargo build -p matching-engine-service -p liquidity-graph-service -p execution-engine -p gateway-service',
    {
      cwd: repoRoot,
      stdio: 'inherit',
      shell: isWin,
      env: cargoEnv,
    },
  );

  console.log('[dev-stack] Starting matching-engine-service (8083)…');
  const match = spawn(matchExe, [], {
    cwd: repoRoot,
    stdio: 'inherit',
    env: process.env,
  });
  children.push(match);

  match.on('exit', (code) => {
    if (code !== 0 && code !== null) {
      console.error('[dev-stack] matching-engine-service exited with code', code);
      killAll();
      process.exit(code);
    }
  });

  try {
    await waitHttpHealth('http://127.0.0.1:8083/health', (t) =>
      jsonHealthMatchesService(t, 'matching-engine-service'),
    );
    console.log('[dev-stack] Matching engine is up (HTTP /health OK).');
  } catch (e) {
    console.error('[dev-stack]', e instanceof Error ? e.message : e);
    killAll();
    process.exit(1);
  }

  console.log('[dev-stack] Starting liquidity-graph-service (8091)…');
  const liq = spawn(liqExe, [], {
    cwd: repoRoot,
    stdio: 'inherit',
    env: process.env,
  });
  children.push(liq);
  liq.on('exit', (code) => {
    if (code !== 0 && code !== null) {
      console.error('[dev-stack] liquidity-graph-service exited with code', code);
      killAll();
      process.exit(code);
    }
  });
  try {
    await waitHttpHealth('http://127.0.0.1:8091/health', (t) =>
      jsonHealthMatchesService(t, 'liquidity-graph-service'),
    );
    console.log('[dev-stack] Liquidity graph service is up (HTTP /health OK).');
  } catch (e) {
    console.error('[dev-stack]', e instanceof Error ? e.message : e);
    killAll();
    process.exit(1);
  }

  console.log('[dev-stack] Starting execution-engine (8092)…');
  const engineProc = spawn(execExe, [], {
    cwd: repoRoot,
    stdio: 'inherit',
    env: process.env,
  });
  children.push(engineProc);
  engineProc.on('exit', (code) => {
    if (code !== 0 && code !== null) {
      console.error('[dev-stack] execution-engine exited with code', code);
      killAll();
      process.exit(code);
    }
  });
  try {
    await waitHttpHealth('http://127.0.0.1:8092/health', (t) =>
      jsonHealthMatchesService(t, 'execution-engine'),
    );
    console.log('[dev-stack] Execution engine is up (HTTP /health OK).');
  } catch (e) {
    console.error('[dev-stack]', e instanceof Error ? e.message : e);
    killAll();
    process.exit(1);
  }

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
    NEXT_PUBLIC_API_URL: `http://127.0.0.1:${gwPort}`,
  };

  console.log(
    `[dev-stack] Starting gateway-service (port ${gwPort}; set GATEWAY_HTTP_PORT to pin a port)…`,
  );
  const gw = spawn(gwExe, [], {
    cwd: repoRoot,
    stdio: 'inherit',
    env: stackEnv,
  });
  children.push(gw);

  gw.on('exit', (code) => {
    if (code !== 0 && code !== null) {
      console.error(
        '[dev-stack] gateway-service exited with code',
        code,
        '\nPort',
        gwPort,
        'failed to bind — unset GATEWAY_HTTP_PORT to auto-pick 8080–8099, or free the port.',
      );
      killAll();
      process.exit(code);
    }
  });

  try {
    await waitHttpHealth(`http://127.0.0.1:${gwPort}/health`, (t) =>
      jsonHealthMatchesService(t, 'gateway'),
    );
    console.log('[dev-stack] Gateway is up (HTTP /health OK).');
  } catch (e) {
    console.error('[dev-stack]', e instanceof Error ? e.message : e);
    killAll();
    process.exit(1);
  }

  console.log('[dev-stack] Starting Next.js…');
  const ui = spawn('npm', ['run', 'dev'], {
    cwd: uiRoot,
    stdio: 'inherit',
    shell: isWin,
    env: stackEnv,
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
