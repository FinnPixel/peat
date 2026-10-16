[![CI](https://github.com/FinnPixel/peat/actions/workflows/ci.yml/badge.svg)](https://github.com/FinnPixel/peat/actions/workflows/ci.yml)

# peat

Small CLI tool that checks your browser bookmarks for dead links.

Point it at an exported bookmarks file to know which URLs are still alive, which are gone, and which have moved.

## Example

```
$ peat bookmarks.html
342/342 checked
------------ REPORT ------------
Total: 342 URLs.

not found (4):
  https://example.com/old-post (404)

blocked (2):
  https://1password.com/ (403)

moved (1):
  https://www.notion.so/ → https://www.notion.com/

alive: 335
```

## Install

```sh
git clone https://github.com/FinnPixel/peat.git
cd peat
cargo install --path .
```

```sh
peat <path-to-bookmarks.html>   # or a .txt/.md list of URLs
peat --help
```

## Clean up your bookmarks

```sh
peat bookmarks.html --clean cleaned.html
```

This writes a cleaned copy of your bookmarks. It:

- removes only links the server reports as gone (404 / 410)
- updates moved links to their new URL
- keeps everything else: folders, titles, order, and links that are blocked, timed out or erroring
- never overwrites your original file

## Run without installing

```sh
cargo run --release -- <path-to-bookmarks.html>
```
