# git-backup

A tool to backup your git repositories from one of the following:

- Github
- GitLab
- Bitbucket

You use this tool by pointing at the source you'd like to perform the backup from, a destination folder to place backups, and a `--token` (which can be provided as an environmental variable) which provides access to the cloud source (necessary for obtaining a list of repositories and for backing up private repositories using the `git` CLI tool).

Example usage:

```
# make the access token available to the tool:
export GIT_TOKEN=youraccesstokenhere

# backing up all repositories from github
# (all of the below and more are fine):
git-backup github/jsdw ~/path/to/backups
git-backup https://github.com/jsdw ~/path/to/backups
git-backup git@github.com/jsdw ~/path/to/backups

# backing up all repositories from gitlab
# (similar formats to the above are accepted):
git-backup gitlab/jsdw ~/path/to/backups

# backing up all repositories from bitbucket
# (similar formats to the above are accepted):
git-backup bitbucket/jsdw ~/path/to/backups

# backing up a single repository from somewhere
# (all of the below and more are fine):
git-backup github/jsdw/my-repo ~/path/to/backups
git-backup github.com/jsdw/my-repo ~/path/to/backups
git-backup github.com/jsdw/my-repo.git ~/path/to/backups
git-backup git@github.com:jsdw/my-repo.git ~/path/to/backups
```

## Obtaining a token

You'll need a token which you can provide using `--token` or the environment variable `GIT_TOKEN` in order to use this tool. This token will be used to obtain a list of repositories to backup (including private ones) and give the `git` CLI tool access to your repositories to clone/sync them locally.

Here's how to get a token depending on the service you wish to backup from:

### GitLab

In GitLab, you'll need to create a new *Personal Access Token* with the `api` scope.

Navigate to *Settings -> Access Tokens* to create one, and you'll need to tick the `api` scope.

### Github

Github also has a notion of a *Personal Access Token*.

Navigate to *Settings -> Developer Settings -> Personal Access Tokens -> Generate new token*. You'll need to tick the `repo` scope.

### Bitbucket

Bitbucket has a concept called *App passwords*, which is what you'll need to provide to this backup tool.

To obtain one, navigate to *Profile -> Settings -> App passwords -> Create App Password*. Tick the `read` scope under the `Repositories` heading.

# Install from source

In this repository root, run:

```
cargo --install --path .
```

To install a binry into your `PATH`. You may need to add `--force` if you have already installed a rust binary (for example, a prior version of this tool) with the same name.