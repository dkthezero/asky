# PRD: Providers Management (Tab 3)

## Overview
Providers translate raw managed logical code (Skills and Instructions natively fetched from Vaults) into target systems tailored exclusively for proprietary deployment structures (such as mapping YAML outputs exclusively for specific Claude Desktop protocols, or placing code within Letta constraints). 

The Provider functionality serves as a bridging abstraction that ensures AI agents can leverage tools downloaded from generic repositories natively within their discrete tool interfaces.

## Functional Requirements
- **Simultaneous Target Output Ecosystems:** `agk` relies on the design ability to switch active configurations for multiple target AI frameworks simultaneously. Users can select and broadcast changes to Copilot and Claude locally in parallel.
- **Native Implementation Rollout:** Provide built-in support explicitly mapping these platforms:
  - GitHub Copilot
  - Firebender
  - Claude Desktop
  - Letta
  - Snowflake Cortex
  - Gemini CLI
  - AMP Code
  - Claude Code
- **Scope Targeting Overrides:** Switch active providers dependently defined upon Global (`[global]`) scaling scopes or Workspace (`[workspace]`) root implementations. A workspace setup overriding generic scopes may alter default tooling injections natively (like dropping Copilot skills implicitly to the `$PWD/.github/skills` directory inherently instead of `~/.copilot/skills/`). 
- **Tab 3 UI Details:**
  - Render simple, boolean configurations mapped exclusively using standard spacing events to broadcast intent changes visually to the user.
  - Active marker symbols highlighting which integrations are actively synchronizing package instructions implicitly based on background tasks.
