# Tildr man

The `.md` files in this directory are for generating the `man tiller` command.

## Auto (recommended)

To automatically generate the `man` documentation, use the command:

```sh
make man
```

## Manual

To generate the `man` documentation manually, use the command:

```sh
pandoc -s -t man docs/man/tildr*.md -o docs/man/dist/tildr*.1
```

To compress the `man` files into `.gz` files, use the following command:

```sh
gzip -f docs/man/dist/tildr*.1
```
