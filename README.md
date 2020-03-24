# `facts` - [Factorio server](https://www.factorio.com/download-headless) management CLI

Easily create and manage multiple servers. Automatically update to latest factorio.

## Usage

### Quickstart

Create and run a world with default settings with

`facts create ExampleWorld && facts start ExampleWorld`

### Usage examples

#### Create new server

`facts create ExampleWorld`

* `--factorio experimental` to use the experimental versíon
* `--factorio 0.16` to force latest `0.16`
* `--map-gen-settings map-gen-settings.json` to specify map generation settings
* `--map-settings map-settings.json` to specify map settings
* `--server-settings map-settings.json` to specify server settings
* `--server-adminlist server-adminlist.json` to specify server admin list
* `--add-admin AdminUserName` to add server admins
* `--autoupdate SETTING`
  * `enabled` automatically apply updates when no players are online (default)
  * `forced` immediately restart when updates are available, kicking out players
  * `startup` auto-update on server startup
  * `disabled` never auto-update

#### Import existing world to facts

`facts import ExampleWorld world.zip`

Supports all arguments from `facts create`, except map(-gen)-settings which cannot be changed after creation.


#### Edit server settings

`facts edit ExampleWorld`

Supports all arguments from `facts create`, except map(-gen)-settings which cannot be changed after creation.

#### Switch server to use the experimental build

`facts edit ExampleWorld --factorio experimental`

#### Start server

`facts start ExampleWorld`

#### Export (back up) a world.zip from facts

`facts export ExampleWorld world.zip`

#### Delete a server (requires confirmation)

`facts delete ExampleWorld`

* `--force` to skip confirmation prompt

#### List all servers

`facts list`

#### Remove unused server versions

`facts prune`

#### Update facts itself (not implemented yet)

`facts self update`
