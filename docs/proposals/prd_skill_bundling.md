# PRD: Skill Bundling & Meta-Skills

> **Product Mindset:** `agk` is the agent kit for teams to share the way they work with AI agents together. Meta-skills are the ultimate team onboarding tool — one identifier deploys an entire standardized workflow.

---

## Overview

Currently, users must install individual skills one by one. A meta-skill (or "skill pack") is a single `SKILL.md` that declares a list of dependencies. When installed, `agk` recursively resolves and installs the entire dependency tree. This enables tech leads to create standardized "way of work" packages that junior engineers can deploy with a single command.

---

## Functional Requirements

### Dependency Declaration
- Extend `SKILL.md` YAML frontmatter to support a `requires:` array.
- Each entry is an asset identity: `vault/name` or `vault/name:version`.
- Optional: `requires_optional:` for soft dependencies that are installed only if available (no failure if missing).

### Recursive Installation
- When `agk` processes an install for an asset with `requires:`, it resolves each dependency against configured vaults and pushes them onto an installation queue.
- Installation is depth-first, parallel where possible (subject to provider I/O limits).
- The parent meta-skill is considered "installed" only when all required children succeed.

### Circular Dependency Guard
- Track the active installation branch (`Vec<String>` of names).
- If a dependency resolves to a name already in the branch, abort with a clear error:
  ```
  Error: Circular dependency detected
    frontend-engineer-pack
    → react-parser
    → component-generator
    → frontend-engineer-pack (cycle)
  ```

### Diamond Dependency Deduplication
- If two sibling skills both require the same grandchild, the grandchild is installed only once.
- Deduplication is keyed by `(vault, name, sha10)`. If the target sha10 matches the already-installed version, skip.

### Scope Inheritance
- A meta-skill installed in **Global** scope installs its children in **Global** scope.
- A meta-skill installed in **Workspace** scope installs its children in **Workspace** scope.
- No cross-scope leakage.

---

## YAML Parsing Schema

```yaml
---
name: frontend-engineer-pack
version: "1.0.0"
description: "A meta-bundle of skills optimized for React and CSS workflows."
requires:
  - clawhub/react-parser
  - clawhub/css-linter
  - internal-vault/component-generator
requires_optional:
  - clawhub/storybook-scaffold
---
```

---

## User Personas & Expected UX

### 👤 Human User

| Scenario | Expected UX |
|----------|-------------|
| Tech lead creates a pack | Writes a `SKILL.md` with `requires:` and pushes it to the team vault. |
| Junior engineer onboarding | Opens TUI → Skills tab → finds `Acme-Company-Pack` → presses `Space`. TUI shows a macro-progress bar: "Installing pack (3/7)…" with child tasks listed below. |
| Circular dependency encountered | TUI detail pane shows a red error banner with the exact cycle path. Install button is disabled. |
| Diamond dependency | User installs `frontend-engineer-pack`. `react-parser` and `css-linter` both require `dom-utils`. `dom-utils` installs once. Progress bar correctly counts 3/3, not 4/4. |

### 🤖 AI Agent User

| Scenario | Expected UX |
|----------|-------------|
| Agent installs a pack | `agk install acme/frontend-pack --json` returns a tree: `{"pack": "frontend-pack", "installed": 7, "skipped": 1, "failed": 0, "tree": [...]}` |
| Agent validates a pack | `agk validate` checks that all `requires:` entries are resolvable. If a vault is missing, returns exit code `2` with the missing identity. |

### 🏭 CI/CD User

| Scenario | Expected UX |
|----------|-------------|
| Team pack is versioned | `agk validate` in CI ensures the `requires:` list contains no broken or dangling references before the pack is published. |
| Onboarding script | `agk install acme/onboarding-pack --global --quiet` in a team setup script. Exit `0` = new machine is ready. |

---

## Non-Goals
- Semver range resolution (`^1.0.0`, `>=2.0.0`). agk uses `sha10` as the source of truth, not version ranges. The `requires:` array specifies exact identities or relies on vault default resolution.
- Nested meta-skills within meta-skills beyond depth 3 (defensible limit to prevent runaway installs).
- Auto-update of child dependencies when the parent pack is updated. Updating a pack reinstalls children based on the pack's current `requires:` list.

---

## Acceptance Criteria
- [ ] `SKILL.md` frontmatter parser reads `requires:` and `requires_optional:`.
- [ ] Recursive installation uses pure async functions from P1 (Headless CLI).
- [ ] Circular dependencies are detected and rejected with a clear error path.
- [ ] Diamond dependencies are deduplicated by `(vault, name, sha10)`.
- [ ] TUI shows a macro-progress bar for pack installation with child task increments.
- [ ] `--json` output for `agk install` includes the dependency tree.
- [ ] `agk validate` detects unresolvable `requires:` entries.

---

*End of PRD.*
