# oxyn-cli

The `oxyn` command line: add [Oxyn](https://oxynui.com) agent UI components to a Dioxus or GPUI
project. Like shadcn/ui, it copies the component's source into your project (`src/oxyn/`), so the
code is yours to change.

```sh
cargo install oxyn-cli
cd my-dioxus-or-gpui-app
oxyn init                          # oxyn.toml, src/oxyn/, `mod oxyn;` and oxyn-core = "0.1" (crates.io)
oxyn add composer approval-card    # copies the code and the components it needs
```

| Command | What it does |
|---|---|
| `oxyn list` | Every item, free and Pro (`--category`, `--tier`, `--json`) |
| `oxyn search <words>` | Search names, categories and descriptions |
| `oxyn view <item>` | What an item is, its files and dependencies |
| `oxyn add <items>…` | Add items and their dependencies to this project |
| `oxyn diff <item>` | Compare an installed item with the registry |
| `oxyn login` / `logout` / `whoami` | Sign in for Pro items and see this month's count |

74 components are free (MIT). With a [Pro or Team plan](https://oxynui.com/pricing/), `oxyn login`
signs the CLI in and `oxyn add` installs Pro items too. The registry defaults to
`https://oxynui.com/r` (override with `--registry` or `OXYN_REGISTRY`). Full docs:
[oxynui.com/docs/cli](https://oxynui.com/docs/cli/).

## License

MIT. Source, issues and the full component catalogue: [github.com/ismailqureshi72/Oxyn](https://github.com/ismailqureshi72/Oxyn) · [oxynui.com](https://oxynui.com).
