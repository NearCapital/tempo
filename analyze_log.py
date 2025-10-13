#!/usr/bin/env python3
"""
Analyze debug.log to calculate average times for:
- Build finished (payload building)
- State root computation
- Regular root task (from payload job creation to root task finished)
- Block added to canonical chain
"""

import re
from datetime import datetime
from statistics import mean, median, stdev
from pathlib import Path


def parse_time_to_ms(time_str):
    """
    Convert time strings to milliseconds.
    Handles formats like: 1.186125ms, 11.583µs, 2.666375ms, 180.042292ms, etc.
    """
    time_str = time_str.strip()

    # Match patterns like "123.456ms" or "789.012µs" or "1.234s"
    match = re.match(r'([\d.]+)(ms|µs|s)', time_str)
    if not match:
        return None

    value, unit = match.groups()
    value = float(value)

    # Convert to milliseconds
    if unit == 'ms':
        return value
    elif unit == 'µs':
        return value / 1000.0
    elif unit == 's':
        return value * 1000.0

    return None


def strip_ansi_codes(text):
    """Remove ANSI escape codes from text."""
    ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
    return ansi_escape.sub('', text)


def parse_timestamp(line):
    """Extract timestamp from log line."""
    # Strip ANSI codes first
    clean_line = strip_ansi_codes(line)
    # Match ISO timestamp at start of line: 2025-09-29T22:22:37.272569Z
    match = re.match(r'(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)', clean_line)
    if match:
        return datetime.fromisoformat(match.group(1).replace('Z', '+00:00'))
    return None


def parse_log_file(log_file):
    """Parse the debug.log file and extract timing information."""

    build_times = []
    state_root_times = []
    block_added_times = []
    payload_to_received_times = []  # Time from "Built payload" to "Received block"

    # Track "Built payload" times by parent block number
    # We use parent_number because the next block will be parent_number + 1
    built_payload_times = {}  # block_number -> timestamp

    with open(log_file, 'r', encoding='utf-8') as f:
        for line in f:
            timestamp = parse_timestamp(line)
            # Strip ANSI codes for easier pattern matching
            clean_line = strip_ansi_codes(line)

            # Parse "Built payload" lines for build time and track timestamp
            if 'Built payload' in clean_line:
                # Extract parent_number from build_payload{...parent_number=9672...}
                parent_match = re.search(r'parent_number\s*=\s*(\d+)', clean_line)
                if parent_match and timestamp:
                    parent_number = int(parent_match.group(1))
                    block_number = parent_number + 1
                    # Skip block 1
                    if block_number != 1:
                        built_payload_times[block_number] = timestamp

                # Extract elapsed time
                match = re.search(r'elapsed\s*=\s*([\d.]+(?:ms|µs|s))', clean_line)
                if match:
                    time_ms = parse_time_to_ms(match.group(1))
                    if time_ms is not None:
                        build_times.append(time_ms)

            # Parse "Received block from consensus engine" and calculate time from "Built payload"
            elif 'Received block from consensus engine' in clean_line:
                number_match = re.search(r'number\s*=\s*(\d+)', clean_line)
                if number_match and timestamp:
                    block_number = int(number_match.group(1))
                    # Skip block 1
                    if block_number != 1 and block_number in built_payload_times:
                        start_time = built_payload_times[block_number]
                        elapsed_ms = (timestamp - start_time).total_seconds() * 1000
                        payload_to_received_times.append(elapsed_ms)
                        # Clean up to save memory
                        del built_payload_times[block_number]

            # Parse "State root task finished" lines (if present in some logs)
            elif 'State root task finished' in clean_line:
                match = re.search(r'elapsed\s*=\s*([\d.]+(?:ms|µs|s))', clean_line)
                if match:
                    time_ms = parse_time_to_ms(match.group(1))
                    if time_ms is not None:
                        state_root_times.append(time_ms)

            # Parse "Block added to canonical chain" lines
            elif 'Block added to canonical chain' in clean_line:
                match = re.search(r'elapsed\s*=\s*([\d.]+(?:ms|µs|s))', clean_line)
                if match:
                    time_ms = parse_time_to_ms(match.group(1))
                    if time_ms is not None:
                        block_added_times.append(time_ms)

    return build_times, state_root_times, payload_to_received_times, block_added_times


def print_statistics(name, times):
    """Print statistics for a given set of timing measurements."""
    if not times:
        print(f"\n{name}: No data found")
        return

    avg = mean(times)
    med = median(times)
    min_time = min(times)
    max_time = max(times)
    std = stdev(times) if len(times) > 1 else 0

    print(f"\n{name}:")
    print(f"  Count:   {len(times)}")
    print(f"  Average: {avg:.3f} ms")
    print(f"  Median:  {med:.3f} ms")
    print(f"  Min:     {min_time:.3f} ms")
    print(f"  Max:     {max_time:.3f} ms")
    print(f"  Std Dev: {std:.3f} ms")


def main():
    log_file = Path(__file__).parent / 'debug.log'

    if not log_file.exists():
        print(f"Error: {log_file} not found")
        return

    print(f"Analyzing {log_file}...")

    build_times, state_root_times, payload_to_received_times, block_added_times = parse_log_file(log_file)

    print("\n" + "="*60)
    print("LOG ANALYSIS RESULTS")
    print("="*60)

    print_statistics("Build Payload Time", build_times)
    print_statistics("State Root Computation Time (Built Payload -> Received Block)", payload_to_received_times)
    print_statistics("Explicit State Root Task Time", state_root_times)
    print_statistics("Block Added to Canonical Chain Time", block_added_times)

    print("\n" + "="*60)


if __name__ == '__main__':
    main()
