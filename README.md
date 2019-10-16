[![Build Status](https://travis-ci.org/jsdw/git-backup.svg?branch=master)](https://travis-ci.org/jsdw/git-backup)

# git-backup

A tool to backup all of your personal git repositories from one of the following sources:

- GitHub (either repositories or gists)
- GitLab
- Bitbucket

The motivation behind this is that as you acquire more git repositories, and work on different repositories across different machines over a period of time, it's less and less likely that you'll have an uptodate copy of all of your repositories stored in one place (except in the cloud with the git service). This tool makes it easy to obtain a local copy of everything you have on such services so that you can store a backup of them yourself. One use case is running the tool on a self hosted backup server as part of a cron job to maintain uptodate copies of your repositories, or alternatively you might just run it on your local machine periodically to ensure that you have access to the latest version of everything.

To use this tool, you'll need a `token` from the service you want to backup your repositories from (see 'Obtaining a token', below).

# Examples

These examples assume that we've installed this tool to somewhere on our `$PATH` (see 'Installing', below).

First, we'll make our `token` available to `git-backup` (this can also be provided via the `--token` flag):

```sh
export GIT_TOKEN=youraccesstokenhere
```

Now, we can back things up by providing a source and destination, like so:

```sh
# backing up all repositories from github
# (all of the below and more are fine):
git-backup github/jsdw ~/path/to/backups
git-backup git@github.com/jsdw ~/path/to/backups
git-backup https://github.com/jsdw ~/path/to/backups

# backing up all gists from github
# (all of the below and more are fine):
git-backup gist.github/jsdw ~/path/to/backups
git-backup https://gist.github.com/jsdw ~/path/to/backups

# backing up all repositories from gitlab
# (similar formats to the above are accepted):
git-backup gitlab/jsdw ~/path/to/backups

# backing up all repositories from bitbucket
# (similar formats to the above are accepted):
git-backup bitbucket/jsdw ~/path/to/backups
```

You can also use this via the `git` command (just remove the hyphen):

```sh
git backup github/jsdw ~/path/to/backups
```

# Installing

## From pre-built binaries

Prebuilt compressed binaries are available [here](https://github.com/jsdw/git-backup/releases/latest). Download the compressed `.tar.gz` file for your OS/architecture and decompress it (on MacOS, this is automatic if you double-click the downloaded file).

If you like, you can download and decompress the latest release on the commandline. On **MacOS**, run:

```sh
curl -L https://github.com/jsdw/git-backup/releases/download/v0.2.0/git-backup-v0.2.0-x86_64-apple-darwin.tar.gz | tar -xz
```

For **Linux**, run:

```sh
curl -L https://github.com/jsdw/git-backup/releases/download/v0.2.0/git-backup-v0.2.0-x86_64-unknown-linux-musl.tar.gz | tar -xz
```

In either case, you'll end up with a `git-backup` binary in your current folder. The examples assume that you have placed this into your `$PATH` so that it can be called from anywhere.

## From Source

You must have a recent version of `rust` installed (see [rustup](https://rustup.rs/)) to do this.

Given this, just run:

```sh
cargo install --git https://github.com/jsdw/git-backup.git --tag v0.2.0
```

To install the latest released binary into your `PATH`. You may need to add `--force` if you have already installed a rust binary (for example, a prior version of this tool) with the same name.

You can also install the latest `master` branch by cloning this repository and running `cargo install --path .` in its root.

# Obtaining a token

You'll need a token which you can provide using `--token` or the environment variable `GIT_TOKEN` in order to use this tool. This token will be used to obtain a list of repositories to backup (including private ones) and give the `git` CLI tool access to your repositories to clone/sync them locally.

Here's how to get a token depending on the service you wish to backup from:

## GitLab

In GitLab, you'll need to create a new *Personal Access Token* with the `api` scope.

Navigate to *Settings -> Access Tokens* to create one, and you'll need to tick the `api` scope.

## GitHub

GitHub also has a notion of a *Personal Access Token*.

Navigate to *Settings -> Developer Settings -> Personal Access Tokens -> Generate new token*.

For standard repositories you'll need to tick the `repo` scope. If you want to backup all of your gists, you'll also need the `gist` scope (otherwise only public gists will be backed up).

## Bitbucket

Bitbucket has a concept called *App passwords*, which is what you'll need to provide to this backup tool.

To obtain one, navigate to *Profile -> Settings -> App passwords -> Create App Password*. Tick the `read` scope under the `Repositories` heading.
