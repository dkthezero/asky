# Technical Design: Providers

## Overview
Provider mechanics isolate I/O logic and proprietary serialization protocols gracefully beneath Trait abstractions. By routing through `ProviderPort`, `asky` retains functional purity universally—meaning core updates functionally resolve through isolated endpoints mapping natively into separate configuration logic trees.

## Trait Contracts

### ProviderPort Trait
```rust
pub trait ProviderPort: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str; // Human readable CamelCase layout
    fn install(&self, pkg: &ScannedPackage, scope: Scope) -> Result<()>;
    fn remove(&self, identity: &AssetIdentity, kind: &AssetKind, scope: Scope) -> Result<()>;
}
```

Similarly, the overarching abstraction logic acts to shield core components. The previous `ProviderAdapter` design heavily modeled these operations:
```rust
trait ProviderAdapter {
    fn id(&self) -> &str;

    fn resolve_global_root(&self) -> Result<PathBuf>;
    fn resolve_workspace_root(&self, workspace: &Path) -> Result<PathBuf>;

    fn install_skill(&self, pkg: &SkillPackage, scope: Scope) -> Result<InstallResult>;
    fn install_instruction(&self, pkg: &InstructionPackage, scope: Scope) -> Result<InstallResult>;

    fn remove_skill(&self, asset: &InstalledAsset, scope: Scope) -> Result<()>;
    fn remove_instruction(&self, asset: &InstalledAsset, scope: Scope) -> Result<()>;

    fn scan_installed(&self, scope: Scope) -> Result<Vec<InstalledAsset>>;
    fn validate(&self, scope: Scope) -> Result<Vec<ValidationIssue>>;
}
```
**Architecture Rules:**
- The Provider implicitly decides the definitive target placement and physical formatting of assets placed dynamically beneath specific provider boundaries. The Core system strictly refrains from hardcoding system targets statically upstream.
- Adapter components manipulate native OS structures: initiating pure shell file copies, transpiling nested schemas into `.yaml` endpoints (where traditional schemas use folder structure arrays), and creating target directory endpoints implicitly on demand.

## Storage Configurations
The `config.toml` manages tracking logical linkages explicitly storing provider strings inherently linking instances natively into the broader app.
```toml
provider = "provider_a"
providers = ["provider_a", "provider_b"] 
```

## Adding Custom Integrations
1. Create a decoupled module routing inside `src/infra/provider/my_provider.rs` explicitly.
2. Abstract implementations linking logic explicitly against the `ProviderPort` trait natively tracking asset locations.
3. Hook functionality universally relying on standard `infra::provider::common::copy_dir` abstraction helpers for seamless folder duplication natively shielding repetitive serialization tasks automatically.
4. Expose the endpoint publicly via mapping functions directly hooked within `src/infra/provider/mod.rs` and injected actively in the `src/app/bootstrap.rs` framework root builder endpoint natively.
