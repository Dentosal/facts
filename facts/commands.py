import sys
import os.path
import tempfile

import pygressbar

import facts
from facts.exceptions import *
from facts.download import stream as downdload_stream

def create(args):
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

def update(args):
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

def version(args):
    print(facts.Server(args.path).version)

def experimental(args):
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

def start(args):
    server = facts.Server(args.path)
    server.start()

def saves(args):
    if args.q:
        exit(1)

    server = facts.Server(args.path)
    print(server.saves)

def genmap(args):
    raise NotImplementedError
