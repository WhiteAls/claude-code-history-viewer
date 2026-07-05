# Lessons

## 2026-04-26 - Use Docker for project verification

- Trigger: User corrected local verification attempt and requested Docker.
- Rule: For this workspace, run Rust/Node verification inside Docker unless the user explicitly approves host execution.
- Prevention: Inspect [`Dockerfile`](../Dockerfile) and mirror its system dependencies for ad-hoc Docker test commands.

## 2026-04-26 - Use a Playwright skill/MCP path when requested

- Trigger: User corrected an ad-hoc Playwright Docker command and requested using a Playwright skill.
- Rule: When the user explicitly asks to use Playwright, first look for an installed Playwright skill/MCP workflow instead of launching custom browser containers.
- Prevention: If no Playwright skill is installed, state that clearly and use available project/API evidence before proposing installation or alternatives.

## 2026-06-26 - Don't pnpm install into the bind-mounted repo as root

- Trigger: Docker frontend verification ran `pnpm install` with the repo bind-mounted and the container as root, creating root-owned `node_modules/` (301M) and `.pnpm-store/` in the working tree.
- Rule: For ad-hoc Node verification in Docker, do not write deps into the bind-mounted repo. Mount the source `:ro`, or install in a container-internal dir, or at minimum ensure the artifacts are gitignored.
- Prevention: `.pnpm-store` is now in `.gitignore`. Prefer the user's existing setup over recreating deps; the user noted a `.gitignore` entry is enough — don't `rm -rf` regenerable artifacts unless asked.

## 2026-06-26 - Network errors from RU: retry, then offer workarounds and stop

- Trigger: Docker build failed on `corepack install` (registry.npmjs.org timeout) while debian mirrors worked — RKN throttling, not a real outage. Was initially treated as "unreachable".
- Rule: On a network error to an external resource, retry up to 3 times; if still failing, look for workarounds (mirror e.g. `registry.npmmirror.com` / `COREPACK_NPM_REGISTRY`, proxy/`PROXY_URL`, VPN) but present them to the user and stop for confirmation — do not silently switch registries/proxies.
- Prevention: backend-only Rust checks avoid npmjs entirely (rust container + stub `dist/`). See global memory `npm-registry-mirror`.

## 2026-06-26 - Custom Tailwind text-* utilities get dropped by tailwind-merge

- Trigger: Some tool blocks (Read/Write/Edit/Terminal headers) rendered with enlarged text. Custom font-size utilities (`text-2xs/3xs`, `text-pxNN`) defined in `src/index.css` were silently removed by `cn()` whenever a `text-{color}` was merged in the same call — tailwind-merge classed them as conflicting `text-*` and kept only the last (the color), so the element fell back to the 16px base. Blocks without a merged color were unaffected → "only some blocks".
- Rule: Any custom utility sharing a Tailwind class prefix (`text-`, `bg-`, `p-`, …) must be registered with `extendTailwindMerge({ extend: { classGroups: … } })` in `src/lib/utils.ts`, or `cn()` may drop it.
- Prevention: Diagnose font/spacing oddities by inspecting the *computed* class list in the live DOM (Playwright `getComputedStyle` + className), not just the source — the source had the class; the merge removed it. Confirm fixes with `cn("text-pxNN ... text-color")` output.

## 2026-06-26 - Playwright/browser-debug artifacts pollute the repo

- Trigger: Playwright MCP runs wrote `.playwright-mcp/*.yml` and root `app-*.png` screenshots into the working tree.
- Rule: `.playwright-mcp/` is gitignored; save ad-hoc screenshots under it or the scratchpad, not the repo root. Clean up one-off debug images you create.

## 2026-06-26 - Confirm vague UI requests against the user's actual selection

- Trigger: User asked to surface the Terminal block "summary" when collapsed. I assumed "summary" = the shell command and implemented a command chip. They actually meant the Bash `description` field ("Poll interval wait") and corrected me.
- Rule: For ambiguous UI requests ("the summary", "that thing", "this label"), don't guess which DOM/data field is meant. Ask the user to point at it (Playwright `window.getSelection()` reads their highlight) before building.
- Prevention: Reading the selected node's class chain (`text-px12 text-muted-foreground` inside `bg-tool-terminal/10`) pinpointed the exact field/component to change.

## 2026-06-26 - Check upstream before re-applying local fixes on update

- Trigger: Updating from v1.11.0 to v1.17.1; 4 of 6 locally-modified files had been fixed upstream independently (cline workspace/UTF-8, watcher loop, get_session_subagents route, Dockerfile).
- Rule: Before porting local changes onto a big upstream jump, diff each touched file against upstream and confirm the fix isn't already there. Drop redundant local changes instead of carrying duplicate logic.
- Prevention: Keep a backup patch of the pre-update diff (`tasks/local-fixes-v1.11.0.patch`) so dropped work is recoverable.

## 2026-07-05 - Local features can be superseded by an upstream release too

- Trigger: After rebasing local work onto v1.18.0, two of our features duplicated upstream PRs that landed in that release — OpenCode global-project handling (#432) and a collapsible provider filter (#431). Same pattern as the v1.17.1 update, but for *new features*, not just re-ported fixes.
- Rule: This project moves fast upstream. Before building a feature, `git log upstream/main` / check merged PRs for the same idea; when a fresh release arrives, re-check existing local features against it, not only fixes.
- Prevention (git technique, since `rebase -i` is blocked in this env): to drop non-contiguous commits, first `git branch backup/... HEAD`, then `git reset --hard <upstream-tag>` and `git cherry-pick <keepers…>`. The backup branch preserves the dropped work. Never force-push the rewritten branch without explicit approval.
