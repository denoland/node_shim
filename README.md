# node_shim

This repo contains a shim for the `node` binary that enables existing scripts
containing `node` invocations to run in Deno. It parses all environment
variables and command line arguments from Node.js, and translates them to
Deno's equivalent.

## Usage

Execute code:

```sh
node_shim --eval "console.log('Hello from Deno!')"
```

Execute a script:

```sh
node_shim path/to/script.js
```

Use `node --run` to run a package.json script:

```sh
node_shim --run start
```

Open the REPL:

```sh
node_shim
```