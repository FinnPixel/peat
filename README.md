# peat

A small CLI tool that checks your browser bookmarks for dead links.

Point it at an exported bookmarks file and it reports which URLs still respond
and which don't.

## Install

```sh
git clone https://github.com/FinnPixel/peat.git
cd peat
cargo install --path .
```

```sh
peat <path-to-bookmarks.html>
```

## Run without installing

```sh
cargo run --release -- <path-to-bookmarks.html>
```
