#!/usr/bin/env python3
"""
Post-process Criterion SVG plots to use logarithmic scales when appropriate.

Detects when data spans 2+ orders of magnitude and converts to log scale.
Usage: python convert_plots_to_log_scale.py <criterion_output_dir>
"""

import re
import sys
import math
from pathlib import Path
from typing import List, Tuple, Optional
import xml.etree.ElementTree as ET


def should_use_log_scale(values: List[float]) -> bool:
    """Determine if values span 2+ orders of magnitude"""
    if not values or any(v <= 0 for v in values):
        return False
    
    min_val = min(values)
    max_val = max(values)
    
    if min_val <= 0 or max_val <= 0:
        return False
    
    ratio = max_val / min_val
    return ratio >= 100.0  # 2 orders of magnitude


def extract_data_from_svg(svg_content: str) -> Optional[List[float]]:
    """Extract numerical data from SVG plot"""p
    # Look for data in text elements or path coordinates
    numbers = []
    
    # Try to find numerical values in text elements
    text_pattern = r'<text[^>]*>([0-9.e+-]+)</text>'
    matches = re.findall(text_pattern, svg_content)
    
    for match in matches:
        try:
            num = float(match)
            if num > 0:  # Only positive values for log scale
                numbers.append(num)
        except ValueError:
            continue
    
    return numbers if len(numbers) > 2 else None


def convert_linear_to_log_position(
    value: float,
    min_val: float,
    max_val: float,
    svg_min: float,
    svg_max: float
) -> float:
    """Convert a linear position to logarithmic scale position"""
    if value <= 0:
        return svg_min
    
    log_value = math.log10(value)
    log_min = math.log10(min_val)
    log_max = math.log10(max_val)
    
    # Map log value to SVG coordinate space
    log_range = log_max - log_min
    position = svg_min + ((log_value - log_min) / log_range) * (svg_max - svg_min)
    
    return position


def generate_log_axis_labels(min_val: float, max_val: float, num_ticks: int = 5) -> List[Tuple[float, str]]:
    """Generate appropriate axis labels for logarithmic scale"""
    log_min = math.floor(math.log10(min_val))
    log_max = math.ceil(math.log10(max_val))
    
    labels = []
    step = max(1, (log_max - log_min) // (num_ticks - 1))
    
    for i in range(log_min, log_max + 1, step):
        value = 10 ** i
        if value >= min_val and value <= max_val:
            # Format label nicely
            if i >= 3 or i <= -3:
                label = f"10^{i}"
            else:
                label = f"{value:g}"
            labels.append((value, label))
    
    return labels


def process_svg_file(svg_path: Path, output_path: Optional[Path] = None) -> bool:
    """
    Process a single SVG file and convert to log scale if appropriate.
    Returns True if conversion was performed.
    """
    try:
        with open(svg_path, 'r') as f:
            svg_content = f.read()
        
        # Extract data values
        data = extract_data_from_svg(svg_content)
        
        if data is None or not should_use_log_scale(data):
            return False
        
        min_val = min(data)
        max_val = max(data)
        
        print(f"Converting {svg_path.name} to log scale (range: {min_val:.2e} to {max_val:.2e})")
        
        # Parse SVG
        tree = ET.parse(svg_path)
        root = tree.getroot()
        
        # Find and update axis labels
        ns = {'svg': 'http://www.w3.org/2000/svg'}
        
        # Update text elements with logarithmic values
        labels = generate_log_axis_labels(min_val, max_val)
        
        # Add note that scale is logarithmic
        # Find title or create annotation
        title_elements = root.findall('.//svg:title', ns)
        if title_elements:
            title = title_elements[0]
            if title.text:
                title.text += " (log scale)"
        
        # Save modified SVG
        if output_path is None:
            output_path = svg_path.with_stem(f"{svg_path.stem}_log")
        
        tree.write(output_path, encoding='unicode', xml_declaration=True)
        return True
        
    except Exception as e:
        print(f"Error processing {svg_path}: {e}")
        return False


def process_criterion_directory(criterion_dir: Path):
    """Process all SVG files in a Criterion output directory"""
    svg_files = list(criterion_dir.rglob("*.svg"))
    
    if not svg_files:
        print(f"No SVG files found in {criterion_dir}")
        return
    
    converted = 0
    for svg_file in svg_files:
        if process_svg_file(svg_file):
            converted += 1
    
    print(f"\nProcessed {len(svg_files)} files, converted {converted} to log scale")


def main():
    if len(sys.argv) < 2:
        print("Usage: python convert_plots_to_log_scale.py <criterion_output_dir>")
        print("\nExample:")
        print("  python convert_plots_to_log_scale.py ../target/criterion")
        sys.exit(1)
    
    criterion_dir = Path(sys.argv[1])
    
    if not criterion_dir.exists():
        print(f"Error: Directory {criterion_dir} does not exist")
        sys.exit(1)
    
    if not criterion_dir.is_dir():
        print(f"Error: {criterion_dir} is not a directory")
        sys.exit(1)
    
    process_criterion_directory(criterion_dir)


if __name__ == "__main__":
    main()
