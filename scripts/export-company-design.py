#!/usr/bin/env python3
"""Export named company reference frames from the saved OpenPencil document."""

import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
ARTBOARDS = {
    "01": "design-desktop.png",
    "02": "design-mobile.png",
    "03": "design-team-desktop.png",
    "04": "design-team-mobile.png",
    "05": "design-testimonials-desktop.png",
    "06": "design-reduced-mobile.png",
    "07": "design-menu-mobile.png",
    "09": "design-mixed-desktop.png",
    "10": "design-mixed-mobile.png",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=ROOT / "target/company-evidence")
    args = parser.parse_args()
    source = ROOT / "design/company-site.fig"
    result = subprocess.run(
        ["openpencil", "eval", str(source), "-c",
         "return figma.currentPage.children.map(n => ({id:n.id,name:n.name}));", "--json"],
        check=True, capture_output=True, text=True,
    )
    nodes = {node["name"][:2]: node for node in json.loads(result.stdout)}
    missing = ARTBOARDS.keys() - nodes.keys()
    if missing:
        raise SystemExit(f"Missing company artboards: {', '.join(sorted(missing))}")
    args.out.mkdir(parents=True, exist_ok=True)
    for prefix, filename in ARTBOARDS.items():
        subprocess.run(
            ["openpencil", "export", str(source), "--node", nodes[prefix]["id"],
             "--font-policy", "strict", "--output", str(args.out / filename)],
            check=True,
        )


if __name__ == "__main__":
    main()
