# Pixa launcher

The launcher is not a finished product. This directory is the catalog and download gate that has to exist before a desktop shell can install editors.

## Stack

The validation core is a small Rust library, the same language as the editors. It has no GUI yet. A window that lists the sixteen apps comes after one editor can be installed from a signed manifest. The editors stay independently installable.

## Trust

A manifest entry is installable only when all of these hold:

- The product id is a lowercase `pixa` name, with no path or query characters.
- The version is plain numeric semver, with no prerelease tag.
- The artifact URL is HTTPS on `github.com` and points at `lukeybird/pixaeffects` release downloads for that exact product and version.
- The URL path does not contain `.` or `..` segments.
- Size is present, greater than zero, and at most 8 GiB.
- SHA-256 is 64 hex characters.
- The unsigned manifest body is exactly the bytes covered by an Ed25519 signature from the pinned launcher verifying key.

The signing private key stays off this repository, off GitHub Actions artifacts, and off the website. Key rotation means shipping a new launcher build whose pinned verifying key accepts the next manifest key, then rejecting the old key after that launcher is itself installed. Expired or unsigned manifests are refused. There is no “latest” URL.

## What this does not do yet

It does not download, mount, or launch an editor. It does not talk to the account service. It does not replace the website.
