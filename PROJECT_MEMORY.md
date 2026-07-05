# Project Memory

> Baseline: local `main` rebuilt on upstream **v1.18.0** (2026-07-05).
> History was reorganized: `main` = v1.18.0 + 3 kept commits (queue-operation fix,
> font-size `cn` fix + BashCard summary, docker/gitignore). The two features that
> upstream shipped independently in v1.18.0 were **dropped from `main`** and preserved
> on branch `backup/local-features-2026-07-05`. See "Update to v1.18.0" below.

## Update to v1.18.0 (2026-07-05)

The 6 local commits (authored by als, on top of v1.18.0) were split: 3 kept, 3 dropped
because upstream v1.18.0 already solves the same problems.

| Our feature | Upstream in v1.18.0 | Decision |
|---|---|---|
| OpenCode legacy/global worktree **inference** (`infer_project_worktree`) | **#432** `should_split_global_project` / `global_directory_hash` — splits global into per-directory projects | **Dropped** — rely on #432 |
| **Collapsible provider filter** (`cchv:providerFilterExpanded`, sliders toggle) | **#431** collapsible provider filter panel (`Collapsible`, `projectTree.providerFiltersOpen`) | **Dropped** — rely on #431 |
| queue-operation fix, font-size `cn` fix, BashCard summary, docker/gitignore | not in upstream | **Kept** on `main` |

- Backup branch `backup/local-features-2026-07-05` holds all 6 original commits (both
  dropped features recoverable there).
- `main` was rewritten (reset to v1.18.0 + cherry-pick of the 3 keepers), so it diverged
  from `origin/main` (which still points at the old 6-commit set): `ahead 3, behind 6`.
  Pushing `main` would be a force-push — operator's decision, not done automatically.
- Verified after rebuild: `cargo check`/`clippy -D warnings`, `tsc`, `eslint`, `i18n:validate` all pass.

> The v1.17.1 notes below are historical (superseded by the v1.18.0 rebuild).

## Update to v1.17.1 (2026-06-26)

Local checkout was 113 commits behind. Fast-forwarded to v1.17.1. Of the 6 local
files modified in the prior (v1.11.0) session, **4 became redundant** because
upstream fixed the same bugs independently:

| Prior local fix | Status in v1.17.1 |
|---|---|
| `cline.rs` `workspace` fallback + UTF-8 `chars().take()` | ✅ upstream — `task_cwd()` with tests (#330/#422) |
| `watcher.rs` refresh-loop dedup | ✅ upstream — content-signature approach + tests (#367) |
| `server/handlers.rs` + `mod.rs` `get_session_subagents` route | ✅ upstream (#311) |
| Dockerfile `rust:1.88-bookworm`, corepack order, runtime libs | ✅ upstream (now `rust:1-bookworm`) |
| `opencode.rs` legacy/global worktree inference | ❌ NOT upstream — **re-ported** (still local) |
| `docker-compose.yml` WSL env + auth | local env config — **re-ported** (still local) |

Backup of the original v1.11.0 working diff (`tasks/local-fixes-v1.11.0.patch`) was
**deleted on 2026-07-05** — its contents are fully covered by upstream v1.17.1/v1.18.0
(cline workspace + `ui_messages`/"Code - Insiders" paths, watcher loop #367, subagents
route #311) or preserved on branch `backup/local-features-2026-07-05` (opencode inference).

## Still-local changes (not in upstream)

### 1. OpenCode legacy/global worktree inference — `src-tauri/src/providers/opencode.rs`

- Old OpenCode data here uses a `global` project whose manifest reports `worktree: "/"`,
  but sessions (`storage/session/global/*.json` and SQLite `session.directory`) store
  `/home/user`. Without inference the UI shows the project as `unknown`.
- Fix: `infer_project_worktree()` — when worktree is empty or `/`, infer the real dir
  from sessions (SQLite first, then JSON manifests); only when unambiguous (single dir).
- All legacy/global sessions point to the same `/home/user`, so they stay one project.

### 2. Queue-operation messages (queued user prompts) — `src-tauri/src/commands/session/load.rs` + frontend

- **Bug:** prompts typed into the queue while the agent is busy are stored as
  `type:"queue-operation"` (`operation:"enqueue"`, text in top-level `content`, no
  uuid/parentUuid). The backend hard-excluded `queue-operation` → these user requests
  vanished from the viewer. The frontend "Show System Messages" toggle was dead for
  them (backend removed them before they reached the frontend).
- **Fix (hybrid):**
  - `normalize_queue_entries()` (load.rs): genuine queued prompts (`enqueue` + plain
    text via `is_genuine_user_text`) are promoted to real `user` messages (always
    shown); service entries (`<task-notification>` enqueues, `remove`/`dequeue`/`popAll`)
    stay `queue-operation` and are gated by the frontend toggle. Also threads each entry
    onto the previous message's uuid so it renders inline instead of floating to the tree root.
  - `classify_line_fast` + full-loader filter now let `queue-operation` through
    (`EXCLUDED_MESSAGE_TYPES` left intact, so metadata/stats/export are unchanged).
  - Frontend: `ClaudeMessageNode.tsx` dispatches `queue-operation` → `QueueOperationRenderer`
    (previously not wired into the timeline); `QueueOperationRenderer.tsx` shows full
    content (was truncated to 100 chars).
- Repro session: `~/.claude/projects/-home-user-sregame/6aea14a9-…jsonl` (lines 109, 147
  are queued prompts; assistant "Понял…" at 152 responds to line 147).

### 3. docker-compose.yml — WSL env + local-only no-auth

- `user: "0:0"` + `HOME=/home/cchv` so read-only host mounts (often 0700) are readable
  and history paths resolve under `/home/cchv`.
- VS Code Insiders WSL-server mount for RooCode/Cline:
  `${HOME}/.vscode-server-insiders/data/User/globalStorage:/home/cchv/.config/Code - Insiders/User/globalStorage:ro`
- Port published to loopback only: `127.0.0.1:8080:3727`.
- **New auth system (#384):** a bare `--no-auth` is now refused on a non-loopback bind,
  and the image ENTRYPOINT pins `--host 0.0.0.0` (needed for Docker port forwarding), so
  the local-only "no auth" equivalent is **`--no-auth --allow-unsafe-no-auth`**. Safe
  because the port is bound to 127.0.0.1.
- Auth flag reference (`src-tauri/src/lib.rs`): token = `--token` / `CCHV_TOKEN`;
  account = `--auth-user` + `--auth-password-hash` (or `CCHV_AUTH_USERNAME` /
  `CCHV_AUTH_PASSWORD_HASH`, hash via `--print-password-hash`); `--secure-cookies` /
  `CCHV_SECURE_COOKIES` for HTTPS.

### 4. UI fixes — `src/lib/utils.ts`, `BashCard.tsx`, `ProjectTree/index.tsx`

- **Enlarged text in tool blocks (Read/Write/Edit/Terminal headers, etc.).** Root
  cause: the app defines custom font-size utilities (`text-2xs/3xs`, reactive
  `text-pxNN` from issue #408) in `src/index.css`, but `cn()` (tailwind-merge)
  doesn't know they belong to the `font-size` group. When a header merged a size +
  a color (e.g. `cn(layout.titleText /* text-px12 */, titleClassName /* text-foreground */)`),
  tailwind-merge treated them as conflicting `text-*` and dropped the size — the
  element fell back to the inherited 16px base. Blocks without a merged color kept
  their size, so only *some* blocks looked enlarged.
  - Fix: `extendTailwindMerge` in `src/lib/utils.ts` registers the custom sizes in the
    `font-size` group. One root fix covers the whole app. Verified: `cn("text-px12 …
    text-foreground")` → keeps `text-px12` (default twMerge dropped it).
- **Bash/Terminal summary not visible when collapsed.** `BashCard` showed the Bash
  `description` (the human summary, e.g. "Poll interval wait") only inside
  `Renderer.Content`, so it was hidden when collapsed. Moved it to the header
  `rightContent` (muted, truncated, full text in `title` tooltip; hidden `<sm`). The
  command stays in the body. `Renderer.Header` is always rendered; only
  `Renderer.Content` is gated by the collapse toggle. (Note: the user means the
  `description` field by "summary", not the command — confirmed via their DOM selection.)
- **Provider filter row is collapsible** (`ProjectTree/index.tsx`). The ALL/Aider/…
  provider chip row took a lot of sidebar space. Added a toggle (sliders + chevron) in
  the EXPLORER header; the row is **collapsed by default** and the choice is remembered
  (`localStorage` key `cchv:providerFilterExpanded`, try/catch). When a non-ALL filter is
  active while collapsed, the toggle icon is highlighted so the active filter is
  discoverable. i18n key `project.toggleProviderFilters` (session namespace) added to all
  5 locales; `types.generated.ts` regenerated; `i18n:validate` passes.

## Environment notes

- npmjs.org is throttled by RKN here; Docker frontend build (`corepack install` /
  `pnpm install`) needs a VPN/mirror. User runs a system-wide VPN to build. See global
  memory `npm-registry-mirror`.
- Verify in Docker (per `tasks/lessons.md`). For backend-only checks, a `rust:1-bookworm`
  container with webkit deps + a stub `dist/` runs `cargo check`/`clippy` without npmjs.
  Do NOT `pnpm install` into the bind-mounted repo as root — it leaves root-owned
  `node_modules`/`.pnpm-store` (now gitignored).

## Useful verification commands (server bound at 127.0.0.1:8080)

### Scan OpenCode projects

```bash
python3 - <<'PY'
import json, urllib.request
base='http://127.0.0.1:8080/api'
req=urllib.request.Request(base+'/scan_all_projects',
    data=json.dumps({"activeProviders":["opencode"]}).encode(),
    headers={'Content-Type':'application/json'}, method='POST')
with urllib.request.urlopen(req, timeout=120) as r:
    print(json.load(r))
PY
```

### Load sessions for the OpenCode global project

```bash
curl -sS -X POST http://127.0.0.1:8080/api/load_provider_sessions \
  -H 'Content-Type: application/json' \
  --data '{"provider":"opencode","projectPath":"opencode://global"}'
```

### Scan RooCode/Cline projects

```bash
python3 - <<'PY'
import json, urllib.request
base='http://127.0.0.1:8080/api'
req=urllib.request.Request(base+'/scan_all_projects',
    data=json.dumps({"activeProviders":["cline"]}).encode(),
    headers={'Content-Type':'application/json'}, method='POST')
with urllib.request.urlopen(req, timeout=120) as r:
    print(json.load(r))
PY
```
