# Pantheon Plugin Specification
## Protocol-Based Extension System

**Document**: 13 of 20
**Version**: Genesis
**Status**: Architectural Specification

---

## 0. Philosophy

In Pantheon, a plugin is not a separate concept. A plugin IS a Cell that implements one or more protocols. This means every plugin automatically gets: identity, lifecycle, security, memory, observability — everything a Cell gets.

---

## 1. Plugin Model

### 1.1 What is a Plugin?

A Plugin is a Cell that:
1. Registers with the Plugin Registry
2. Declares which protocols it implements
3. Declares which capabilities it requires
4. Declares which capabilities it provides
5. Is packaged as a signed, immutable bundle

### 1.2 Plugin Types

| Type | Description | Protocol Hooks |
|------|-------------|----------------|
| Protocol Plugin | Implements a new protocol | Protocol negotiation |
| Service Plugin | Provides a system service | Service discovery |
| Agent Plugin | Provides a new agent type | Agent creation |
| Tool Plugin | Provides tools for agents | Tool execution |
| Decorator Plugin | Modifies behavior | Event middleware |
| Interface Plugin | User interface (CLI, web) | UI rendering |
| Storage Plugin | New storage backend | Storage protocol |
| Intelligence Plugin | New LLM provider | Intelligence protocol |
| Scheduler Plugin | New scheduling policy | Scheduling protocol |
| Security Plugin | New security mechanism | Security protocol |

### 1.3 Plugin vs Cell

A plugin is distinguished from a regular cell by:
1. It is published (signed, versioned, discoverable)
2. It can be installed/uninstalled without system restart
3. It declares protocol support
4. It has a manifest with metadata

---

## 2. Plugin Manifest

```yaml
# pantheon-plugin.yaml
manifest_version: 1
plugin:
  id: "org.pantheon.plugins.git"
  name: "Git Integration"
  version: "1.2.3"
  description: "Git version control integration"
  author:
    id: "cell://org.pantheon.official"
    name: "Pantheon Core Team"

  protocols:
    implements:
      - "pantheon.git.v1"
      - "pantheon.storage.v2"
    requires:
      - "pantheon.shell.v1"
      - "pantheon.filesystem.v1"

  capabilities:
    requires:
      - "network.http.read:github.com"
      - "filesystem.write:/repos/*"
    provides:
      - "git.commit.read"
      - "git.commit.write"
      - "git.branch.*"

  hooks:
    - event: "workflow.before_execute"
      handler: "check_git_status"
    - event: "code.after_write"
      handler: "auto_commit"

  resources:
    cpu: "100m"
    memory: "50MB"
    storage: "1GB"

  security:
    sandbox: "semi-trusted"
    network: ["github.com", "gitlab.com"]
    attestation: true
```

---

## 3. Plugin Lifecycle

### 3.1 State Machine

```
[DISCOVERED] → [VERIFIED] → [INSTALLED] → [ENABLED] → [RUNNING]
                  │                            │            │
                  ▼                            ▼            ▼
             [REJECTED]                   [DISABLED]   [FAILED]
                                                         │
                                                         ▼
                                                    [ROLLED_BACK]
```

### 3.2 Lifecycle Stages

| Stage | Description | Actions |
|-------|-------------|---------|
| DISCOVERED | Plugin found in registry | Metadata fetched |
| VERIFIED | Signature and manifest verified | Hash check, dependency check |
| INSTALLED | Code downloaded and stored | Content-addressed storage |
| ENABLED | Plugin registered with system | Capability mapping, hook registration |
| RUNNING | Plugin active and handling events | Normal operation |
| DISABLED | Plugin paused | Hooks disconnected, no new work |
| FAILED | Plugin encountered unrecoverable error | Alert, possible rollback |
| ROLLED_BACK | Plugin returned to previous version | State restored from checkpoint |
| REJECTED | Plugin failed verification | Reason logged |

### 3.3 Installation Process

1. **Discovery**: Plugin found via registry search or explicit reference
2. **Download**: Code bundle downloaded (content-addressed)
3. **Verification**: 
   - Signature verification (author signed)
   - Hash verification (content integrity)
   - Dependency check (required protocols available)
   - Capability analysis (requested capabilities vs. system policy)
   - Sandbox analysis (static analysis of code)
4. **Registration**: Plugin Cell created and registered
5. **Staging**: Plugin deployed to staging partition
6. **Testing**: Plugin tested in staging (automated)
7. **Canary**: Plugin deployed to 5% of relevant Cells
8. **Promotion**: Plugin deployed to all Cells
9. **Monitoring**: Continuous monitoring of plugin health

---

## 4. Plugin SDK

### 4.1 SDK Components

The Plugin SDK provides:
- **Cell API**: Create and manage cells
- **Protocol API**: Implement and consume protocols
- **Event API**: Subscribe to and publish events
- **Memory API**: Access memory systems
- **Tool API**: Define and expose tools
- **UI API**: Create user interface components
- **Configuration API**: Read typed configuration

### 4.2 Supported Languages (Phase 1)

- **Rust**: Primary SDK (low-level, high performance)
- **WebAssembly**: Sandboxed runtime (safety, portability)
- **Python**: Data analysis, ML, scripting

### 4.3 Extension Points

```rust
// Rust SDK example
#[pantheon_plugin]
impl GitPlugin {
    #[pantheon_init]
    fn init(config: PluginConfig) -> Result<Self> { }

    #[pantheon_protocol("pantheon.git.v1")]
    fn handle_git(&self, request: GitRequest) -> Result<GitResponse> { }

    #[pantheon_hook("code.after_write")]
    fn on_code_written(&self, event: CodeEvent) -> Result<()> { }

    #[pantheon_tool]
    fn git_status(&self, path: String) -> Result<GitStatus> { }
}
```

---

## 5. Plugin Registry

### 5.1 Registry Structure

```
Plugin Registry
├── Official (Pantheon-maintained)
├── Verified (Third-party, verified by Pantheon)
├── Community (Published by community)
└── Local (Private, organization-specific)
```

### 5.2 Registry API

| Endpoint | Description |
|----------|-------------|
| `search(query, filters)` | Search for plugins |
| `get(plugin_id, version)` | Get plugin metadata |
| `publish(manifest, bundle)` | Publish new plugin |
| `update(plugin_id, new_version)` | Publish update |
| `deprecate(plugin_id, reason)` | Deprecate plugin |
| `report(plugin_id, issue_type)` | Report issue |

---

## 6. Plugin Isolation

### 6.1 Isolation Levels

| Level | CPU | Memory | Network | FS | Description |
|-------|-----|--------|---------|----|-------------|
| 0 | Shared | Shared | Full | Full | System plugins (highest trust) |
| 1 | Cgroup | Cgroup | Filtered | Restricted | Verified plugins |
| 2 | Cgroup | Cgroup | None | TempFS | Community plugins |
| 3 | WASM | WASM | None | None | Untrusted plugins (sandboxed to WASM) |

### 6.2 Resource Limits

Each plugin has hard resource limits:
- CPU: cap on millicores
- Memory: cap on bytes (OOM-kill if exceeded)
- Network: bandwidth cap, domain allowlist
- Storage: disk quota
- Events: rate limit
- LLM tokens: token limit

---

## 7. Vendor Lock-In Prevention

- Plugins use open protocols
- Plugin code is inspectable
- Plugin state is portable
- Alternative implementations can replace any plugin
- Registry is federated (no single point of control)
- Community fork-ability: any plugin can be forked

---

## 8. Version Compatibility

### 8.1 Semantic Versioning for Plugins

| Change | Version Bump | System Impact |
|--------|-------------|---------------|
| Bug fix (backward compatible) | Patch | Automatic upgrade |
| New feature (backward compatible) | Minor | Canary rollout |
| Breaking change | Major | Coordinated migration |
| Security fix | Patch + hotfix | Urgent rollout |

### 8.2 Multiple Versions

- Multiple versions of a plugin can coexist
- Each Cell uses the version it was created with
- Migration to new versions is explicit
- Breaking changes require protocol version negotiation
