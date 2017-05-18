# Facts

A simple Factorio server manager for headless Factorio servers. It is currently only supperted on x86-64 Linux, as it's the only platform that supports headless Factorio.

## Usage

Usually `facts` is used inside the server folder. Alternatively you can use `--path` to target a specific server folder.


### Check a verson of an existing server

    $ facts version
    0.15.12

### Update a server

    $ facts update
    Checking for updates... [ok]
    Updating to 0.15.12
    Downloading... [ok]
    Applying changes...
    Update successful

### Start a server

    $ facts start

### Start a server with specific map save

    $ facts start example_world

### Create a new server

Creates a folder `factorio_server` that contains a new server with the specified version.

    $ facts create factorio_server experimental
    Fetching version info... [ok]
    Downloading... [ok]
    Server created
