# Facts

A simple Factorio server manager for headless Factorio servers. It is currently only supperted on x86-64 Linux, as it's the only platform that supports headless Factorio. Facts allows you to create, update and modify servers with simple commands.

## Usage

Usually `facts` is used inside the server folder. Alternatively you can use `--path` to target a specific server folder. See `facts -h` for more extensive help.

### Update a server

    $ facts update
    Checking for updates... [ok]
    Updating to 0.15.15
    Downloading... [ok]
    Applying changes...
    Update successful

### Start a server with the most recently used save

    $ facts start

### Start a server with specific map save

    $ facts start example_world

### Create a new server

Creates a folder `factorio_server` that contains a new server with the specified version.

    $ facts create factorio_server --experimental
    Fetching version info... [ok]
    Downloading... [ok]
    Server created

## Issues

If there are any issues (or missing functionality), please [open an issue on GitHub](https://github.com/Dentosal/facts/issues/new). Please include any possible error messages.

## Contributing

If you add any nice features or fix or polish my code, feel free to create a pull request.

### Style guidelines

* This project uses PEP8 syntax conventions, except the line length limit
    * Longer lines are still not a good thing, and should be avoided
* Please docstring all your functions, except for trivial `@property`-methods
* Commit messages start with a capital letter, and do NOT end with a period
