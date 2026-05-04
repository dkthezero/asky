# Technical Design: Skill Bundling & Meta-Skills

## Overview

Meta-skills extend the `SKILL.md` frontmatter with a `requires:` array. When a meta-skill is installed, `agk` recursively resolves and installs its dependencies. This design doc specifies the parsing, resolution, installation queue, and deduplication logic.

## Architecture Rules

1. **Meta-skills are just Skills.** They use the same `AssetKind::Skill`, `ScannedPackage`, and `ProviderPort::install` flow. The only difference is the optional `requires:` field in their frontmatter.
2. **Resolution is vault-scoped.** Each `requires:` entry must specify `vault/name` or be resolvable against all configured vaults.
3. **Cycles are fatal.** A circular dependency aborts the entire installation with a clear error message.
4. **Diamonds are idempotent.** The same dependency reached via multiple paths is installed once.

## Data Schemas

### SkillFrontmatter (Extended)
```rust
#[derive(Debug, Clone, Deserialize)]
struct SkillFrontmatter {
    name: String,
    version: Option<String>,
    description: Option<String>,
    #[serde(default)]
    requires: Vec<String>,        // "vault/name" or "vault/name:version"
    #[serde(default)]
    requires_optional: Vec<String>,
}
```

### DependencyResolution
```rust
#[derive(Debug, Clone)]
struct DependencyResolution {
    identity: AssetIdentity,
    vault_id: String,
    package: ScannedPackage,
    required_by: Vec<String>, // chain of parent names for error reporting
}
```

### InstallQueue
```rust
#[derive(Debug)]
struct InstallQueue {
    items: Vec<DependencyResolution>,
    visited: HashSet<(String, String, String)>, // (vault_id, name, sha10)
    branch_stack: Vec<String>, // active path for cycle detection
}
```

## Internal Workflows

### Frontmatter Parsing
1. After discovering a `SKILL.md`, read the YAML frontmatter.
2. If `requires:` is present, validate each entry format.
3. Store the `requires:` list on `ScannedPackage` (new optional field).

### Dependency Resolution (Recursive)
```
function resolve(pkg, branch_stack, visited, queue):
    if pkg.name in branch_stack:
        return Err(CircularDependency)
    
    branch_stack.push(pkg.name)
    
    for req in pkg.requires:
        if req is already in visited with same sha10:
            continue
        
        resolved = find_in_vaults(req)
        if not found and req is optional:
            continue
        if not found:
            return Err(MissingDependency)
        
        queue.push(resolved)
        visited.insert((resolved.vault_id, resolved.name, resolved.sha10))
        
        // Recursively resolve the dependency's own requirements
        if resolved.has_requires:
            resolve(resolved, branch_stack, visited, queue)?
    
    branch_stack.pop()
```

### Installation Order
1. Resolve the top-level meta-skill.
2. Build the `InstallQueue` via depth-first resolution.
3. Iterate the queue in order.
4. For each item: call `app::actions::install_asset()` for each active provider.

### TUI Integration
- When a meta-skill is selected and `Space` is pressed:
  - Send a single `TaskStarted` for the pack.
  - As each child completes, send `TaskProgress` with percentage = (completed / total) * 100.
  - The detail pane shows the dependency tree.

## Trait Contracts

No new traits. Uses existing:
- `FeatureSetPort` (for scanning SKILL.md)
- `VaultPort` (for resolving dependencies)
- `ProviderPort` (for installing)
- `ConfigStorePort` (for tracking installed state)

## Module Structure

```
src/infra/feature/skill.rs     # Extend frontmatter parsing
src/app/
  actions.rs                  # Add resolve_dependencies(), install_with_deps()
  bundling.rs                 # NEW: Dependency resolution, queue management, cycle detection
```

## Testing Strategy

- **Unit tests:**
  - Cycle detection with 2-node and 3-node cycles
  - Diamond deduplication (A→B→D, A→C→D should install D once)
  - Optional dependency missing (should not fail)
  - Optional dependency present (should install)
- **Integration tests:**
  - `agk install meta-pack --dry-run` shows the full queue
  - `agk install meta-pack --json` returns the tree

---

*End of Technical Design.*
