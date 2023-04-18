# Cypher Sheet Discord Bot

This Discord bot is a first attempt at integrating the sharing features in
the [cypher_sheet](https://github.com/kwiesmueller/cypher_sheet) app with
Discord.

The bot will read any files shared in channels it has access to and check if
they match the cypher_sheet mimeType and
[`SharedObject` proto](https://github.com/kwiesmueller/cypher_sheet_protos/blob/46d39a4d4e30155d7a3952044407c32dbaf57064/character.proto#L230).

If a file matches the bot will reply to the message that shared the file with an
embed render of the object inside.

As the bot currently requires the permission to read messages it is restricted
by Discord policies to max 100 servers.
This means that for wider deployment a better integration has to be found.

## Planned Features

* Better integration that doesn't require permission to read messages.
* Actions on the Embed that allow players to "Claim" objects indicating they
  are no longer available.

## Deploy

This repository comes with a deploy tool that helps to build and manage
projects, including this one.

To build the container image for this project run:

```sh
export IMAGE_REGISTRY=<your image registry> # default: quay.io/kwiesmueller
cargo run --bin deploy -- image discord-bot
```

To deploy the discord bot on a Kubernetes cluster run:

```sh
export DISCORD_TOKEN=<token for the discord bot>
cargo run --bin deploy -- deploy discord-bot
```

To check the status of your deployment run:

```sh
export DISCORD_TOKEN=<token for the discord bot>
cargo run --bin deploy -- status discord-bot
```

All resources will be created in a new namespace called `cypher-sheet`.

Environment varibles like `DISCORD_TOKEN` are read from an `.env` file in the
repository root, if present.

The deployment tool will use the local git as its source for versions
(image tag). It will use the closest tag + a commit hash if there have been
changes since.
