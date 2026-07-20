# Tildr man

The `.md` files in this directory are for generating the `man tildr` command.

## Auto (recommended)

To automatically generate the `man` documentation, use the command:

```sh
make man
```

## Manual

To generate one `man` page manually, use the command:

```sh
pandoc -s -t man docs/man/tildr.md -o docs/man/dist/tildr.1
```

Repeat the command for each Markdown source, changing both the input and output file names.

To compress the `man` files into `.gz` files, use the following command:

```sh
gzip -f docs/man/dist/tildr*.1
```
