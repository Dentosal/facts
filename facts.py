#!/usr/bin/env python3
import sys
import os.path
import argparse
import tempfile

try:
    import requests
except ImportError:
    print("IE: requests")
    exit(1)

try:
    import pygressbar
except ImportError:
    print("IE: pygressbar")
    exit(1)

import facts
from facts.exceptions import *
from facts.download import stream as downdload_stream

def cmd_create(args):
    if os.path.exists(args.path):
        if not args.q:
            print("Error: Folder already exists")
        exit(2)
    else:
        if args.q:
            ver = facts.version.latest(args.experimental)
            facts.Server.create(args.path, ver)
        else:
            print("Fetching version info... ", end="")
            sys.stdout.flush()

            ver = facts.version.latest(args.experimental)

            print("[ok]")

            with pygressbar.PercentageProgressBar(width=30, show_value=True) as pb:
                print("Downloading... ", end="")
                sys.stdout.flush()

                pb.update(0)
                def callback(current, total):
                    pb.update(current, update_max_value=total)

                facts.Server.create(args.path, ver, dl_callback=callback)

                print("[ok]")
            print("Server created")

def cmd_update(args):
    server = facts.Server(args.path)
    if args.experimental:
        server.settings.experimental = True

    if args.q:
        server.update(experimental)

    else:
        # NOTE: Duplicating facts.Server.update to allow easier printing
        # TODO: This should probably be refactored

        print("Checking for updates... ", end="")
        sys.stdout.flush()

        update_path = server.get_update_path(server.settings.experimental)

        print("[ok]")

        if update_path == []:
            print("Already the newest version")
        else:
            with tempfile.TemporaryDirectory() as td:
                with pygressbar.MultiProgressBar([
                    pygressbar.ValueProgressBar(len(update_path), width=30, show_value=True),
                    pygressbar.PercentageProgressBar(width=30, show_value=True)
                ]) as pbs:

                    file_downloaded = 0

                    print("Updating to {}".format(update_path[-1].new))

                    print("Downloading... ", end="")
                    sys.stdout.flush()

                    def dl_callback(cursor, size):
                        global file_downloaded
                        file_downloaded = cursor
                        pbs.bars[1].max_value = size

                    pbs.update(0, file_downloaded)
                    for i, update in enumerate(update_path):
                        with open(os.path.join(td, update.tempfilename), "wb") as tf:
                            downdload_stream(update.link, tf, dl_callback)
                        pbs.update(i+1, file_downloaded)

                    print("[ok]")

                print("Applying changes...")

                with pygressbar.ValueProgressBar(len(update_path), width=30, show_value=True) as pb:
                    pb.update(0)
                    for i, update in enumerate(update_path):
                        update_error = server.apply_update(os.path.join(td, update.tempfilename))
                        if update_error:
                            print(update_error)

                        pb.update(i+1)

                print("Update successful")

def cmd_version(args):
    print(facts.Server(args.path).version)

def cmd_experimental(args):
    server = facts.Server(args.path)
    if args.action == "show":
        if args.q:
            exit(0 if server.settings.experimental else 1)
        else:
            print("enabled" if server.settings.experimental else "disabled")
    elif args.action == "enable":
        server.settings.experimental = True
    elif args.action == "disable":
        server.settings.experimental = False

def cmd_start(args):
    server = facts.Server(args.path)
    server.start()

def cmd_saves(args):
    if args.q:
        exit(1)

    server = facts.Server(args.path)
    print(server.saves)

def cmd_genmap(args):
    raise NotImplementedError

def main():
    global_flags = argparse.ArgumentParser(add_help=False)
    global_flags.add_argument("-q", action='store_true', help="Surpressess all output")

    optional_path = argparse.ArgumentParser(add_help=False)
    optional_path.add_argument("--path", default=".", help="Path for the server directory")

    parser = argparse.ArgumentParser(description="Factorio server helper", parents=[global_flags])
    subparsers = parser.add_subparsers(dest="command")
    subparsers.required = True

    # Command: create
    create_parser = subparsers.add_parser("create", parents=[global_flags])
    create_parser.add_argument("path", help="Path for the server directory")
    create_parser.add_argument("--experimental", action="store_true", help="Use experimental version")
    create_parser.set_defaults(func=cmd_create)

    # Command: update
    update_parser = subparsers.add_parser("update", parents=[global_flags, optional_path])
    update_parser.add_argument("--experimental", action="store_true", help="Use experimental version")
    update_parser.set_defaults(func=cmd_update)

    # Command: version
    version_parser = subparsers.add_parser("version", parents=[global_flags, optional_path])
    version_parser.set_defaults(func=cmd_version)

    # Command: experimental
    experimental_parser = subparsers.add_parser("experimental", parents=[global_flags, optional_path])
    experimental_parser.add_argument("action", choices=["show", "enable", "disable"], default="show")
    experimental_parser.set_defaults(func=cmd_experimental)

    # Command: start
    start_parser = subparsers.add_parser("start", parents=[global_flags, optional_path])
    start_parser.add_argument("save", help="Save to use (defaults to the most recently used one)")
    start_parser.set_defaults(func=cmd_start)

    # Command: saves
    saves_parser = subparsers.add_parser("saves", help="List all saved games", parents=[global_flags, optional_path])
    saves_parser.set_defaults(func=cmd_saves)

    # Command: genmap
    genmap_parser = subparsers.add_parser("genmap", help="List all saved games", parents=[global_flags, optional_path])
    genmap_parser.add_argument("name", help="name for the new save")
    genmap_parser.set_defaults(func=cmd_genmap)

    args = parser.parse_args()
    try:
        args.func(args)
    except requests.exceptions.ConnectTimeout:
        if not args.q:
            print("Connection timed out")
        exit(3)

if __name__ == '__main__':
    main()
