#!/usr/bin/env python3
"""Merge Cobertura reports and probe a production file's coverage.

The coverage gate measures the regular and e2e scopes in separate
instrumentation runs and needs one combined report. `cargo llvm-cov` writes a
Cobertura file per run; this tool unions the per-file line hits (the same
semantics as a profile merge) and recomputes the rates the gate reads.

Usage:
    cobertura.py merge <a.xml> <b.xml> <out.xml>
    cobertura.py probe <report.xml> <filename>...
"""

from __future__ import annotations

import sys
import time
import xml.etree.ElementTree as ET


def _classes(root: ET.Element) -> dict[str, ET.Element]:
    classes: dict[str, ET.Element] = {}
    for cls in root.iter("class"):
        filename = cls.get("filename")
        if filename:
            classes[filename] = cls
    return classes


def _lines(cls: ET.Element) -> dict[int, ET.Element]:
    lines: dict[int, ET.Element] = {}
    for line in cls.findall("./lines/line"):
        number = line.get("number")
        if number is not None:
            lines[int(number)] = line
    return lines


def _int(value: str | None, default: int = 0) -> int:
    try:
        return int(value or default)
    except ValueError:
        return default


def _hit_map(cls: ET.Element) -> dict[int, int]:
    return {number: _int(line.get("hits")) for number, line in _lines(cls).items()}


def merge(paths: list[str], out_path: str) -> None:
    merged_classes: dict[str, tuple[str, dict[int, int]]] = {}
    sources: list[str] = []
    for path in paths:
        root = ET.parse(path).getroot()
        for source in root.findall("./sources/source"):
            text = (source.text or "").strip()
            if text and text not in sources:
                sources.append(text)
        for filename, cls in _classes(root).items():
            name, lines = merged_classes.get(filename, (cls.get("name", ""), {}))
            if cls.get("name"):
                name = cls.get("name", name)
            for number, hits in _hit_map(cls).items():
                lines[number] = lines.get(number, 0) + hits
            merged_classes[filename] = (name, lines)

    package = ET.Element(
        "package",
        {"name": "benchmark", "line-rate": "0", "branch-rate": "0", "complexity": "0"},
    )
    classes_el = ET.SubElement(package, "classes")
    total_valid = 0
    total_covered = 0
    for filename in sorted(merged_classes):
        name, hits_map = merged_classes[filename]
        lines = sorted(hits_map.items())
        covered = sum(1 for hits in hits_map.values() if hits > 0)
        total_valid += len(lines)
        total_covered += covered
        rate = f"{covered / len(lines):.6f}" if lines else "0"
        cls_el = ET.SubElement(
            classes_el,
            "class",
            {"name": name, "filename": filename, "line-rate": rate, "branch-rate": "0"},
        )
        lines_el = ET.SubElement(cls_el, "lines")
        for number, hits in lines:
            ET.SubElement(
                lines_el,
                "line",
                {"number": str(number), "hits": str(hits), "branch": "false"},
            )
    if total_valid:
        package.set("line-rate", f"{total_covered / total_valid:.6f}")

    coverage = ET.Element(
        "coverage",
        {
            "line-rate": f"{total_covered / total_valid:.6f}" if total_valid else "0",
            "branch-rate": "0",
            "lines-valid": str(total_valid),
            "lines-covered": str(total_covered),
            "branches-valid": "0",
            "branches-covered": "0",
            "complexity": "0",
            "version": "merged",
            "timestamp": str(int(time.time())),
        },
    )
    sources_el = ET.SubElement(coverage, "sources")
    for source in sources or ["."]:
        ET.SubElement(sources_el, "source").text = source
    packages_el = ET.SubElement(coverage, "packages")
    packages_el.append(package)
    ET.ElementTree(coverage).write(out_path, encoding="utf-8", xml_declaration=True)


def probe(report_path: str, filenames: list[str]) -> int:
    root = ET.parse(report_path).getroot()
    classes = _classes(root)
    status = 0
    for filename in filenames:
        cls = classes.get(filename)
        if cls is None:
            matches = [name for name in classes if name.endswith(filename)]
            cls = classes[matches[0]] if matches else None
        if cls is None:
            print(f"coverage probe: {filename}: no class in {report_path}", file=sys.stderr)
            status = 1
            continue
        lines = _lines(cls)
        covered = sum(1 for line in lines.values() if _int(line.get("hits")) > 0)
        hits = sum(_int(line.get("hits")) for line in lines.values())
        print(f"coverage probe: {filename}: covered_lines={covered} hits={hits}")
        if covered == 0:
            status = 1
    return status


def main(argv: list[str]) -> int:
    if len(argv) >= 5 and argv[1] == "merge":
        merge(argv[2:4], argv[4])
        return 0
    if len(argv) >= 4 and argv[1] == "probe":
        return probe(argv[2], argv[3:])
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
