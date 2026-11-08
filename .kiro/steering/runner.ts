/**
 * Adapter entry point — registers the Kiro runner.
 *
 * This is the thin shim the orchestrator (or an adapter loader) calls to
 * make the `kiro` runner available in a {@link RunnerRegistry}. It mirrors
 * `adapters/claude-code/runner.ts` exactly so Kiro is a first-class adapter,
 * not merely a runner class buried in `src/runners/`.
 *
 * All behavior lives in `src/runners/kiro.ts` ({@link KiroRunner}); this file
 * only wires the runner into a registry and honors the existing injectable
 * {@link KiroRunnerOptions} seam (bin / execFileFn / spawnFn) so tests can
 * register a fully-faked runner without spawning `kiro-cli`.
 *
 * @see docs/rfc/runner-bridge-contract.md §5.3 (kiro)
 * @see adapters/claude-code/runner.ts (the parity model)
 */

import {
  KiroRunner,
  RunnerRegistry,
  type KiroRunnerOptions,
  type Runner,
} from '../../src/runners';

/**
 * Construct a Kiro runner instance.
 *
 * @param opts - optional injectable dependencies (bin / execFileFn / spawnFn);
 *               omitting them gives the real `child_process` functions and the
 *               default `kiro-cli` binary.
 * @returns a fresh {@link KiroRunner}.
 */
export function createKiroRunner(opts?: KiroRunnerOptions): Runner {
  return opts ? new KiroRunner(opts) : new KiroRunner();
}

/**
 * Register the Kiro runner with a {@link RunnerRegistry}.
 *
 * Detection is intentionally left to the caller: the registry runs
 * `detect()` for every registered runner via {@link RunnerRegistry.detect}.
 *
 * @param registry - the registry to register into.
 * @param opts     - optional injectable dependencies override.
 * @returns the registered runner instance.
 */
export function registerKiroRunner(
  registry: RunnerRegistry,
  opts?: KiroRunnerOptions,
): Runner {
  const runner = createKiroRunner(opts);
  registry.register(runner);
  return runner;
}

/** Stable id of the runner this adapter registers. */
export const KIRO_RUNNER_ID = 'kiro';

export default registerKiroRunner;
