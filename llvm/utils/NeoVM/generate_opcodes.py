#!/usr/bin/env python3

"""Extract NeoVM opcode metadata from the upstream Neo runtime sources.

This utility consumes the C# enum definition in `neo/src/Neo.VM/OpCode.cs`
and the matching gas schedule in
`neo/src/Neo/SmartContract/ApplicationEngine.OpCodePrices.cs` to produce a
JSON payload that captures:

  * opcode name and byte value
  * operand size metadata (fixed width or length-prefix)
  * stack push/pop counts inferred from the XML doc comments
  * default gas price in Neo's ApplicationEngine

Downstream build steps can translate the emitted JSON into TableGen records.

The script intentionally avoids executing arbitrary expressions from the
source files; it only understands the `1 << n` pattern used in the gas table
plus plain integer literals.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Tuple


NEO_ROOT = Path(__file__).resolve().parents[3] / "neo"
OPCODE_SOURCE = NEO_ROOT / "src" / "Neo.VM" / "OpCode.cs"
PRICE_SOURCE = (
    NEO_ROOT
    / "src"
    / "Neo"
    / "SmartContract"
    / "ApplicationEngine.OpCodePrices.cs"
)


@dataclass
class OperandInfo:
    size: Optional[int] = None
    size_prefix: Optional[int] = None


@dataclass
class OpcodeRecord:
    name: str
    value: int
    push: Optional[int]
    pop: Optional[int]
    operand: OperandInfo
    gas: Optional[int]
    summary: Optional[str]


_CONST_RE = re.compile(r"^(?P<name>[A-Z0-9_]+)\s*=\s*(?P<value>0x[0-9A-Fa-f]+),?$")
_OPERAND_RE = re.compile(r"\[OperandSize\(([^)]*)\)\]")
_OPERAND_FIELD_RE = re.compile(r"(Size|SizePrefix)\s*=\s*(\d+)")
_PUSH_RE = re.compile(r"Push:\s*(\d+)")
_POP_RE = re.compile(r"Pop:\s*(\d+)")
_PRICE_RE = re.compile(r"\[OpCode\.(?P<name>[A-Za-z0-9_]+)\]\s*=\s*(?P<expr>[^,]+),")


def _parse_operand(line: str) -> OperandInfo:
    match = _OPERAND_RE.search(line)
    info = OperandInfo()
    if not match:
        return info
    for field, value in _OPERAND_FIELD_RE.findall(match.group(1)):
        if field == "Size":
            info.size = int(value)
        elif field == "SizePrefix":
            info.size_prefix = int(value)
    return info


def _extract_push_pop(comment_block: Iterable[str]) -> Tuple[Optional[int], Optional[int], Optional[str]]:
    text = "\n".join(comment_block)
    push = None
    pop = None
    summary_lines: List[str] = []
    for line in comment_block:
        cleaned = line.strip()
        if cleaned.startswith("///"):
            payload = cleaned[3:].strip()
            if payload.startswith("<summary>"):
                continue
            if payload.startswith("</summary>"):
                continue
            if payload.startswith("<remarks>") or payload.startswith("</remarks>"):
                continue
            if payload:
                summary_lines.append(payload)
    push_match = _PUSH_RE.search(text)
    pop_match = _POP_RE.search(text)
    if push_match:
        push = int(push_match.group(1))
    if pop_match:
        pop = int(pop_match.group(1))
    summary = " ".join(summary_lines).strip() or None
    return push, pop, summary


def parse_opcodes(path: Path) -> Dict[str, OpcodeRecord]:
    records: Dict[str, OpcodeRecord] = {}
    pending_comments: List[str] = []
    pending_operand = OperandInfo()

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.rstrip()
        stripped = line.strip()

        if stripped.startswith("///"):
            pending_comments.append(stripped)
            continue

        if "[OperandSize" in stripped:
            pending_operand = _parse_operand(stripped)
            continue

        const_match = _CONST_RE.match(stripped)
        if const_match:
            name = const_match.group("name")
            value = int(const_match.group("value"), 16)
            push, pop, summary = _extract_push_pop(pending_comments)
            records[name] = OpcodeRecord(
                name=name,
                value=value,
                push=push,
                pop=pop,
                operand=pending_operand,
                gas=None,
                summary=summary,
            )
            pending_comments = []
            pending_operand = OperandInfo()
            continue

        # Reset documentation context when hitting region markers or blanks.
        if not stripped:
            pending_comments = []
            pending_operand = OperandInfo()

    return records


def parse_prices(path: Path) -> Dict[str, int]:
    prices: Dict[str, int] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        match = _PRICE_RE.search(line)
        if not match:
            continue
        name = match.group("name")
        expr = match.group("expr").strip()
        if expr.startswith("1 <<"):
            shift = int(expr.split("<<", 1)[1])
            value = 1 << shift
        else:
            value = int(expr, 0)
        prices[name] = value
    return prices


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        "-o",
        type=Path,
        help="Optional JSON output file. Defaults to stdout when omitted.",
    )
    parser.add_argument(
        "--emit-td",
        type=Path,
        help="Optional path to write an auto-generated TableGen fragment.",
    )
    parser.add_argument(
        "--emit-bytes",
        type=Path,
        help="Optional path to write a C++ helper mapping opcodes to byte values.",
    )
    parser.add_argument(
        "--neo-root",
        type=Path,
        default=NEO_ROOT,
        help="Root of the Neo repository (auto-detected by default).",
    )
    args = parser.parse_args()

    opcode_path = args.neo_root / "src" / "Neo.VM" / "OpCode.cs"
    price_path = (
        args.neo_root
        / "src"
        / "Neo"
        / "SmartContract"
        / "ApplicationEngine.OpCodePrices.cs"
    )

    if not opcode_path.is_file():
        raise FileNotFoundError(f"Unable to locate OpCode.cs at {opcode_path}")
    if not price_path.is_file():
        raise FileNotFoundError(
            f"Unable to locate ApplicationEngine.OpCodePrices.cs at {price_path}"
        )

    records = parse_opcodes(opcode_path)
    prices = parse_prices(price_path)

    for name, gas in prices.items():
        if name in records:
            records[name].gas = gas

    serialisable = {
        name: {
            **asdict(record),
            "operand": {k: v for k, v in asdict(record.operand).items() if v is not None},
        }
        for name, record in sorted(records.items(), key=lambda item: item[1].value)
    }

    payload = json.dumps(serialisable, indent=2, sort_keys=False)
    if args.output:
        args.output.write_text(payload + "\n", encoding="utf-8")
    elif not args.emit_td:
        print(payload)

    if args.emit_td:
        td_lines: List[str] = []
        td_lines.append("// Auto-generated by generate_opcodes.py --emit-td")
        td_lines.append("// Source: neo/src/Neo.VM/OpCode.cs")
        td_lines.append("")
        td_lines.append("class NeoVMInst<int opcode, int push, int pop, int gas> : Instruction {")
        td_lines.append("  bits<8> Inst;")
        td_lines.append("  let Namespace = \"NeoVM\";")
        td_lines.append("  let OutOperandList = (outs);")
        td_lines.append("  let InOperandList = (ins);")
        td_lines.append("  let Size = 0; // Variable by default; overridden per instruction when known")
        td_lines.append("  let TSFlags = !or(push, !or(!shl(pop, 8), !shl(gas, 16)));")
        td_lines.append("  let Inst = opcode;")
        td_lines.append("}")
        td_lines.append("")

        for record in serialisable.values():
            opcode = record["value"]
            push = record.get("push") or 0
            pop = record.get("pop") or 0
            gas = record.get("gas") or 0
            operand = record.get("operand", {})
            size = operand.get("size")
            size_prefix = operand.get("size_prefix")

            if size_prefix is not None:
                size_expr = "0"  # variable length because prefix controls runtime size
            elif size is not None:
                size_expr = str(1 + size)
            else:
                size_expr = "1"

            td_lines.append(
                f"def {record['name']} : NeoVMInst<0x{opcode:02X}, {push}, {pop}, {gas}> {{"
            )
            td_lines.append(f"  let Size = {size_expr};")
            td_lines.append(f"  let AsmString = \"{record['name'].lower()}\";")
            td_lines.append("}")
            td_lines.append("")

        args.emit_td.parent.mkdir(parents=True, exist_ok=True)
        args.emit_td.write_text("\n".join(td_lines), encoding="utf-8")

    if args.emit_bytes:
        args.emit_bytes.parent.mkdir(parents=True, exist_ok=True)
        with args.emit_bytes.open("w", encoding="utf-8") as out:
            out.write("// Auto-generated by generate_opcodes.py --emit-bytes\n")
            out.write("// Source: neo/src/Neo.VM/OpCode.cs\n\n")
            out.write("#pragma once\n\n")
            out.write("#include <cstdint>\n")
            out.write("#include <optional>\n\n")
            out.write("namespace llvm {\nnamespace NeoVMEncoding {\n\n")
            out.write("struct OpcodeEncoding {\n")
            out.write("  uint8_t Byte = 0;\n")
            out.write("  uint8_t OperandSize = 0;\n")
            out.write("  uint8_t OperandPrefixSize = 0;\n")
            out.write("  bool HasImmediate = false;\n")
            out.write("  bool HasSizePrefix = false;\n")
            out.write("};\n\n")
            out.write(
                "inline std::optional<OpcodeEncoding> getOpcodeEncoding(unsigned Opcode) {\n"
            )
            out.write("  switch (Opcode) {\n")
            for record in serialisable.values():
                operand = record.get("operand", {})
                size = operand.get("size") or 0
                prefix = operand.get("size_prefix") or 0
                has_size = size or prefix
                out.write(
                    f"  case NeoVM::{record['name']}: return OpcodeEncoding{{0x{record['value']:02X}, {size}, {prefix}, {'true' if has_size else 'false'}, {'true' if prefix else 'false'}}};\n"
                )
            out.write("  default: return std::nullopt;\n  }\n}\n\n")
            out.write("} // namespace NeoVMEncoding\n} // namespace llvm\n")


if __name__ == "__main__":
    main()
