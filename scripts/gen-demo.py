#!/usr/bin/env python3
"""
Generate an asciinema v2 cast file for tablefmt.

Usage:
    python3 scripts/gen-demo.py
    agg docs/demo/tablefmt-demo.cast docs/demo/tablefmt-demo.gif
"""

import json
import subprocess
import sys
import os

BINARY = "./target/debug/tablefmt"
OUTPUT = "docs/demo/tablefmt-demo.cast"
WIDTH = 100
HEIGHT = 30
TITLE = "tablefmt – terminal table formatter"

PROMPT_COLOR = "\x1b[32m"  # green
RESET = "\x1b[0m"
BOLD = "\x1b[1m"

CHARS_PER_SEC = 28.0
PAUSE_AFTER_PROMPT = 0.6   # pause before typing starts
PAUSE_AFTER_CMD = 0.4      # pause between enter and output
PAUSE_AFTER_OUTPUT = 2.5   # pause to read output


class CastWriter:
    def __init__(self, path, width, height, title):
        self.path = path
        self.header = {"version": 2, "width": width, "height": height,
                       "timestamp": 0, "title": title}
        self.events = []
        self.t = 0.0

    def _out(self, text):
        self.events.append([round(self.t, 4), "o", text])

    def pause(self, seconds):
        self.t += seconds

    def _type(self, text):
        delay = 1.0 / CHARS_PER_SEC
        for ch in text:
            self._out(ch)
            self.t += delay

    def newline(self):
        self._out("\r\n")

    def prompt(self):
        self._out(f"\r\n{PROMPT_COLOR}${RESET} ")
        self.pause(PAUSE_AFTER_PROMPT)

    def section_header(self, text):
        self._out(f"\r\n{BOLD}# {text}{RESET}\r\n")
        self.pause(0.8)

    def run(self, display_cmd, actual_args):
        """Show display_cmd in terminal but run actual_args for real output."""
        self.prompt()
        self._type(display_cmd)
        self._out("\r\n")
        self.pause(PAUSE_AFTER_CMD)

        env = os.environ.copy()
        env["NO_COLOR"] = "1"
        result = subprocess.run(actual_args, capture_output=True, text=True, env=env)
        output = result.stdout
        if result.returncode != 0:
            output += result.stderr

        # Normalize newlines for terminal
        output = output.replace("\n", "\r\n")
        if output:
            self._out(output)
        self.pause(PAUSE_AFTER_OUTPUT)

    def write(self):
        os.makedirs(os.path.dirname(self.path), exist_ok=True)
        with open(self.path, "w", encoding="utf-8") as f:
            f.write(json.dumps(self.header) + "\n")
            for event in self.events:
                f.write(json.dumps(event, ensure_ascii=False) + "\n")
        print(f"wrote {self.path}  ({len(self.events)} events, {self.t:.1f}s)")


def check_binary():
    if not os.path.exists(BINARY):
        print(f"binary not found at {BINARY}, building…", file=sys.stderr)
        subprocess.run(["cargo", "build"], check=True)


def main():
    check_binary()

    w = CastWriter(OUTPUT, WIDTH, HEIGHT, TITLE)

    w.pause(0.5)
    w.section_header("CSV input → GitHub-flavored Markdown (default)")
    w.run(
        "tablefmt format -i docs/demo/data/sales.csv",
        [BINARY, "format", "-i", "docs/demo/data/sales.csv"],
    )

    w.section_header("Same data, PostgreSQL style")
    w.run(
        "tablefmt format -i docs/demo/data/sales.csv -S psql",
        [BINARY, "format", "-i", "docs/demo/data/sales.csv", "-S", "psql"],
    )

    w.section_header("Unicode box-drawing style")
    w.run(
        "tablefmt format -i docs/demo/data/sales.csv -S modern",
        [BINARY, "format", "-i", "docs/demo/data/sales.csv", "-S", "modern"],
    )

    w.section_header("JSON input")
    w.run(
        "tablefmt format -i docs/demo/data/products.json",
        [BINARY, "format", "-i", "docs/demo/data/products.json"],
    )

    w.section_header("YAML input → reStructuredText table")
    w.run(
        "tablefmt format -i docs/demo/data/metrics.yaml -S rst",
        [BINARY, "format", "-i", "docs/demo/data/metrics.yaml", "-S", "rst"],
    )

    w.section_header("Re-align an existing Markdown table with prettify")
    w.run(
        "tablefmt prettify -S github -i docs/demo/data/table.md",
        [BINARY, "prettify", "-S", "github", "-i", "docs/demo/data/table.md"],
    )

    w.pause(1.0)
    w.write()


if __name__ == "__main__":
    main()
