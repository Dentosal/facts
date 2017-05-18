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
            ver = facts.resolve_version_name(args.version)
            facts.Server.create(args.path, ver)
        else:
            print("Fetching version info... ", end="")
            sys.stdout.flush()

            ver = facts.resolve_version_name(args.version)

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

    # NOTE: Duplicating facts.Server.update to allow easier printing
    # TODO: This should probably be refactored

    print(hasattr(args, "experimental"))

    print("Checking for updates... ", end="")
    sys.stdout.flush()

    update_path = server.get_update_path(True)

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

def main():
    global_flags = argparse.ArgumentParser(add_help=False)
    global_flags.add_argument("-q", action='store_true', help="Surpressess all output")

    parser = argparse.ArgumentParser(description="Factorio server helper", parents=[global_flags])
    subparsers = parser.add_subparsers(dest="command")
    subparsers.required = True

    # Command: create
    create_parser = subparsers.add_parser("create", parents=[global_flags])
    create_parser.add_argument("path", help="Path for the server directory")
    create_parser.add_argument(
        "version",
        help="Either 'stable', 'experimental' or in numeric format (default: experimental)",
        nargs="?",
        default="experimental"
    )
    create_parser.set_defaults(func=cmd_create)

    # Command: update
    update_parser = subparsers.add_parser("update", parents=[global_flags])
    update_parser.add_argument("path", help="Path for the server directory")
    update_parser.add_argument(
        "version",
        help="Either 'stable', 'experimental' or in numeric format (default: experimental)",
        nargs="?",
        default="experimental"
    )
    update_parser.set_defaults(func=cmd_update)

    # Command: version
    version_parser = subparsers.add_parser("version", parents=[global_flags])
    version_parser.add_argument("path", help="Path for the server directory")
    version_parser.set_defaults(func=cmd_version)

    args = parser.parse_args()
    try:
        args.func(args)
    except requests.exceptions.ConnectTimeout:
        if not args.q:
            print("Connection timed out")
        exit(3)

if __name__ == '__main__':
    main()
