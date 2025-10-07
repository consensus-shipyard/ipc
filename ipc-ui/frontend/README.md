# IPC Web UI

This is the web-based user interface for InterPlanetary Consensus (IPC), built with Vue 3, Vite, and TypeScript.

## Recommended IDE Setup

[VSCode](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur).

## Type Support for `.vue` Imports in TS

TypeScript cannot handle type information for `.vue` imports by default, so we replace the `tsc` CLI with `vue-tsc` for type checking. In editors, we need [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) to make the TypeScript language service aware of `.vue` types.

## Development Setup

### Project Setup

```sh
npm install
```

### Compile and Hot-Reload for Development

```sh
npm run dev
```

This will start a development server with hot module replacement for rapid development.

### Type-Check, Compile and Minify for Production

```sh
npm run build
```

### Lint with [ESLint](https://eslint.org/)

```sh
npm run lint
```

## Building into IPC CLI

The UI is designed to be embedded into the `ipc-cli` binary. This allows the CLI to serve the web interface without requiring separate deployment or setup.

### Building the CLI with UI Support

From the **root of the IPC repository** (not from this frontend directory), run:

```sh
make build-with-ui
```

This command will:
1. Build the frontend Vue.js application (this project)
2. Compile the frontend assets into optimized production files
3. Embed the compiled UI assets into the `ipc-cli` Rust binary
4. Generate the final binary at `./target/release/ipc-cli`

### Starting the UI from IPC CLI

Once built with UI support, you can start the embedded web server:

```sh
./target/release/ipc-cli ui
```

The UI will be accessible at `http://localhost:3030` by default. The command output will show the exact URL.

### How It Works

The build process:
1. Runs `npm run build` in this directory to create production assets in the `dist/` folder
2. The Rust build process embeds these static files into the `ipc-cli` binary
3. When you run `ipc-cli ui`, it starts an embedded web server that serves these assets
4. The UI communicates with IPC through the CLI's backend services

## Customize configuration

See [Vite Configuration Reference](https://vite.dev/config/).

## Features

The IPC UI provides a graphical interface for:
- Managing and viewing subnets
- Monitoring subnet status and validators
- Interacting with IPC operations
- Configuring and managing IPC settings

For more information about IPC, visit the [main IPC repository](https://github.com/consensus-shipyard/ipc) or the [IPC project page](https://www.ipc.space/).