# Ruff Doctor Examples

## Generic checks

```bash
ruff doctor
```

## JSON output

```bash
ruff doctor --json
```

## Deep mode

```bash
ruff doctor --deep
```

## Profile discovery

```bash
ruff doctor --list-profiles
```

## Profile execution

When profile contributions are installed/registered:

```bash
ruff doctor wordpress
ruff doctor --profile vercel
```

## Canonical workflow-pack path

```bash
ruff pack run doctor doctor
ruff pack run doctor doctor --json
```
