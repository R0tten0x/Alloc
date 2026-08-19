#!/usr/bin/env python3
"""Generate memory-chip menubar icons (SVG + PNG) in red/yellow/green."""
import subprocess
from pathlib import Path

OUT = Path(__file__).parent

COLORS = {
    "red":    {"fill": "#E5484D", "stroke": "#8C1D22", "pin": "#B23238"},
    "yellow": {"fill": "#F5C518", "stroke": "#8A6A0A", "pin": "#C79A12"},
    "green":  {"fill": "#3DBE5B", "stroke": "#1E6B33", "pin": "#2E9647"},
}

def pins(x_side, y_positions, direction):
    """direction: 1 = pins point right (left edge), -1 = pins point left (right edge)"""
    out = []
    for y in y_positions:
        x2 = x_side + (4 * direction)
        out.append(f'<line x1="{x_side}" y1="{y}" x2="{x2}" y2="{y}" />')
    return "\n    ".join(out)

def pins_vert(y_side, x_positions, direction):
    out = []
    for x in x_positions:
        y2 = y_side + (4 * direction)
        out.append(f'<line x1="{x}" y1="{y_side}" x2="{x}" y2="{y2}" />')
    return "\n    ".join(out)

TEMPLATE = """<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">
  <g fill="none" stroke="{pin}" stroke-width="1.6" stroke-linecap="square">
    {pins_left}
    {pins_right}
    {pins_top}
    {pins_bottom}
  </g>
  <rect x="7" y="7" width="18" height="18" rx="2.2"
        fill="{fill}" stroke="{stroke}" stroke-width="1.4"/>
  <circle cx="10.3" cy="10.3" r="1.15" fill="{stroke}" opacity="0.55"/>
  <g fill="none" stroke="{stroke}" stroke-width="1.1" stroke-linecap="round" opacity="0.85">
    <line x1="10.5" y1="13.5" x2="21.5" y2="13.5"/>
    <line x1="10.5" y1="16" x2="21.5" y2="16"/>
    <line x1="10.5" y1="18.5" x2="21.5" y2="18.5"/>
    <line x1="10.5" y1="21" x2="17" y2="21"/>
  </g>
</svg>
"""

Y_LEFT = [11, 16, 21]
Y_RIGHT = [11, 16, 21]
X_TOP = [12, 16, 20]
X_BOTTOM = [12, 16, 20]

for name, c in COLORS.items():
    svg = TEMPLATE.format(
        fill=c["fill"], stroke=c["stroke"], pin=c["pin"],
        pins_left=pins(7, Y_LEFT, -1),
        pins_right=pins(25, Y_RIGHT, 1),
        pins_top=pins_vert(7, X_TOP, -1),
        pins_bottom=pins_vert(25, X_BOTTOM, 1),
    )
    svg_path = OUT / f"memory-chip-{name}.svg"
    svg_path.write_text(svg)
    png_path = OUT / f"memory-chip-{name}.png"
    subprocess.run(["rsvg-convert", "-w", "32", "-h", "32", str(svg_path), "-o", str(png_path)], check=True)
    # @2x for retina menubars
    png2_path = OUT / f"memory-chip-{name}@2x.png"
    subprocess.run(["rsvg-convert", "-w", "64", "-h", "64", str(svg_path), "-o", str(png2_path)], check=True)

print("done")
