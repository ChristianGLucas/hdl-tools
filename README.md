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
