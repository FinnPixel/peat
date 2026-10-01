[![CI](https://github.com/FinnPixel/peat/actions/workflows/ci.yml/badge.svg)](https://github.com/FinnPixel/peat/actions/workflows/ci.yml)

# peat

Small CLI tool that checks your browser bookmarks for dead links.

Point it at an exported bookmarks file to know which URLs are still alive.

## Install

```sh
git clone https://github.com/FinnPixel/peat.git
cd peat
cargo install --path .
```

```sh
peat <path-to-bookmarks.html>   # or a .txt/.md list of URLs
```

## Clean up your bookmarks

```sh
peat bookmarks.html --clean cleaned.html
```

This writes a copy of your bookmarks with dead links removed and moved links pointing at their new URL. Folders, titles, order, and blocked links stay as they are.

## Run without installing

```sh
cargo run --release -- <path-to-bookmarks.html>
```
