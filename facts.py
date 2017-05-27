#!/usr/bin/env python3
import argparse

# Import external dependencies to test that they are installed
import requests
import pygressbar

import facts.commands as cmd

def main():
    optional_path = argparse.ArgumentParser(add_help=False)
    optional_path.add_argument("--path", default=".", help="Path for the server directory")

    parser = argparse.ArgumentParser(description="Factorio server helper", parents=[])
    parser.add_argument("-q", action='store_true', help="Surpressess all output")
    subparsers = parser.add_subparsers(dest="command")
    subparsers.required = True

    # Command: create
    create_parser = subparsers.add_parser("create", parents=[])
    create_parser.add_argument("path", help="Path for the server directory")
    create_parser.add_argument("--experimental", action="store_true", help="Use experimental version")
    create_parser.set_defaults(func=cmd.create)

    # Command: update
    update_parser = subparsers.add_parser("update", parents=[optional_path])
    update_parser.add_argument("--experimental", action="store_true", help="Use experimental version")
    update_parser.set_defaults(func=cmd.update)

    # Command: version
    version_parser = subparsers.add_parser("version", parents=[optional_path])
    version_parser.set_defaults(func=cmd.version)

    # Command: experimental
    experimental_parser = subparsers.add_parser("experimental", parents=[optional_path])
    experimental_parser.add_argument("action", choices=["show", "enable", "disable"], default="show")
    experimental_parser.set_defaults(func=cmd.experimental)

    # Command: start
    start_parser = subparsers.add_parser("start", parents=[optional_path])
    start_parser.add_argument("save", help="Save to use (defaults to the most recently used one)")
    start_parser.set_defaults(func=cmd.start)

    # Command: saves
    saves_parser = subparsers.add_parser("saves", help="List all saved games", parents=[optional_path])
    saves_parser.set_defaults(func=cmd.saves)

    # Command: genmap
    genmap_parser = subparsers.add_parser("genmap", help="List all saved games", parents=[optional_path])
    genmap_parser.add_argument("name", help="name for the new save")
    genmap_parser.set_defaults(func=cmd.genmap)

    args = parser.parse_args()
    try:
        args.func(args)
    except requests.exceptions.ConnectTimeout:
        if not args.q:
            print("Connection timed out")
        exit(3)

if __name__ == '__main__':
    main()
