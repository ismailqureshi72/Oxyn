# oxyn-registry-schema

The data types of the [Oxyn](https://oxynui.com) component registry: the registry index (every item,
free and Pro, with its kind, category, tier and frameworks) and a registry item (its files and
dependencies). Shared by the registry builder, the `oxyn` CLI (`oxyn-cli`) and the MCP server
(`oxyn-mcp`), so all three read the registry at `https://oxynui.com/r` the same way.

```toml
[dependencies]
oxyn-registry-schema = "0.1"
```

## License

MIT. Source, issues and the full component catalogue: [github.com/ismailqureshi72/Oxyn](https://github.com/ismailqureshi72/Oxyn) · [oxynui.com](https://oxynui.com).
