# hdl-tools

Deterministic structural parsing and interface inspection for Hardware
Description Language (HDL) source — Verilog, SystemVerilog and VHDL. Built for
the Axiom marketplace (`christiangeorgelucas/hdl-tools`).

FPGA/ASIC design agents work with HDL source as text: they need to know what a
module or entity's interface looks like — its ports, directions, types, bit
widths, and parameters/generics — without synthesizing or simulating it. This
package answers that: given HDL source text, it detects the dialect and
extracts a structured interface description.

Distinct from `christiangeorgelucas/code-parse-tools` (general-purpose
multi-language tree-sitter access) — this package does HDL-specific
*structured interface extraction*: modules/entities, ports with
direction+type+width, parameters/generics, doc comments, and ready-to-copy
instantiation templates, not a generic syntax-tree passthrough.

## Use it from your agent or app

Every node in this package is a **live, auto-scaling API endpoint** on the
[Axiom](https://axiomide.com) marketplace — call it from an AI agent or your own
code, with nothing to self-host.

**📦 See it on the marketplace:**
https://dev.axiomide.com/marketplace/christiangeorgelucas/hdl-tools@0.1.0

**Hook it up to an AI agent (MCP).** Add Axiom's hosted MCP server to any MCP
client and every node becomes a typed tool your agent can call — search the
catalog, inspect a schema, and invoke it directly.

```bash
# Claude Code
claude mcp add --transport http axiom https://api.axiomide.com/mcp \
  --header "Authorization: Bearer $AXIOM_API_KEY"
```

Claude Desktop, Cursor, or any config-based client:

```json
{
  "mcpServers": {
    "axiom": {
      "type": "http",
      "url": "https://api.axiomide.com/mcp",
      "headers": { "Authorization": "Bearer YOUR_AXIOM_API_KEY" }
    }
  }
}
```

**Call it from the CLI.**

```bash
axiom invoke christiangeorgelucas/hdl-tools/DetectLanguage --input '{ ... }'
```

**Call it over HTTP.**

```bash
curl -X POST https://api.axiomide.com/invocations/v1/nodes/christiangeorgelucas/hdl-tools/0.1.0/DetectLanguage \
  -H "Authorization: Bearer $AXIOM_API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{ ... }'
```

> Input/output schema for each node is on the marketplace page above, or via
> `axiom inspect node christiangeorgelucas/hdl-tools/DetectLanguage`.

### Get started free

Install the CLI:

```bash
# macOS / Linux — Homebrew
brew install axiomide/tap/axiom

# macOS / Linux — install script
curl -fsSL https://raw.githubusercontent.com/AxiomIDE/axiom-releases/main/install.sh | sh
```

**Windows:** download the `windows/amd64` `.zip` from the
[releases page](https://github.com/AxiomIDE/axiom-releases/releases), unzip it,
and put `axiom.exe` on your `PATH`.

Then `axiom version` to verify, `axiom login` (GitHub or Google) to authenticate,
and create an API key under **Console → API Keys**. Docs and sign-up at
**[axiomide.com](https://axiomide.com)**.

## What it wraps

Two MIT-licensed, actively maintained tree-sitter grammars do the actual
parsing:

- [`tree-sitter-systemverilog`](https://github.com/gmlarumbe/tree-sitter-systemverilog) —
  a full IEEE 1800-2017 SystemVerilog grammar (Verilog is a strict subset, so
  one grammar covers both dialects).
- [`tree-sitter-vhdl`](https://github.com/jpt13653903/tree-sitter-vhdl) — a
  VHDL grammar that parses entity/port/generic declarations directly (not
  only the legacy component-mirror idiom some older tools require).

This package's own code walks the resulting syntax tree to pull out the
interface-level facts an integration agent needs. It is a pure, deterministic,
offline text-in/structured-data-out transform: no synthesis, no simulation, no
network access, no wall-clock, no randomness.

## Nodes

| Node | What it does |
|---|---|
| `DetectLanguage` | Detect Verilog/SystemVerilog vs VHDL by comparing parse-error counts across both grammars. |
| `ParseVerilogModules` | Parse every module in Verilog/SystemVerilog source (ports + parameters). |
| `ParseVhdlEntities` | Parse every entity in VHDL source (ports + generics). |
| `ParseVhdlPackages` | Parse every VHDL package's types/subtypes/constants/function+procedure signatures. |
| `ListDefinitions` | List every module/entity name defined in a source. |
| `ExtractDefinition` | Extract one named module/entity's full interface. |
| `ExtractPorts` | Extract a module/entity's full port list. |
| `ExtractInputPorts` | Extract only the input-direction ports. |
| `ExtractOutputPorts` | Extract only the output-direction ports. |
| `ExtractParameters` | Extract parameters (Verilog) / generics (VHDL) with type + default. |
| `ExtractDocComments` | Extract doc comments attached to a module/entity and its ports. |
| `Summarize` | Compact counts: modules/entities/packages/ports/parameters. |
| `GenerateInstantiationTemplate` | A ready-to-copy instantiation snippet ("how do I wire this up"). |
| `ValidateSyntax` | Confirm source parses cleanly; list every syntax issue found. |

## License

MIT — Copyright (c) 2026 Christian George Lucas.
