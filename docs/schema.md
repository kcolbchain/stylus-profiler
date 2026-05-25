# JSON Report Schema

## Schema Version

Current: `1.0.0`

### Version Bump Policy

- **Major bump** (1.x -> 2.x): breaking field removal, type change, or semantic change
- **Minor bump** (1.1 -> 1.2): additive field additions only
- **Patch bump** (1.0.1 -> 1.0.2): cosmetic/docs changes, no field changes

## Sample Output

```json
{
  "schema_version": "1.0.0",
  "binary_size_bytes": 65536,
  "total_code_size": 48000,
  "total_functions": 45,
  "functions": [
    {
      "index": 0,
      "name": "main",
      "body_size": 12000,
      "instruction_count": 3000,
      "estimated_gas": 4500,
      "suggestion": null
    }
  ],
  "sections": [
    {
      "name": "code",
      "size_bytes": 48000,
      "kind": "code"
    },
    {
      "name": "data",
      "size_bytes": 8192,
      "kind": "data"
    },
    {
      "name": "name",
      "size_bytes": 2048,
      "kind": "custom"
    }
  ]
}
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `schema_version` | string | Semver of the report schema |
| `binary_size_bytes` | int | Raw WASM binary file size |
| `total_code_size` | int | Sum of all function body sizes |
| `total_functions` | int | Total function count |
| `functions` | array | Top-level function breakdown (sorted by body_size desc) |
| `sections` | array | WASM sections with size attribution |

## Function Object

| Field | Type | Description |
|-------|------|-------------|
| `index` | int | Function index in the WASM module |
| `name` | string | Function name (or `func_N` if unnamed) |
| `body_size` | int | Raw byte size of the function body |
| `instruction_count` | int | Number of WASM instructions |
| `estimated_gas` | int | Estimated Stylus gas cost |
| `suggestion` | string or null | Optimization hint, if applicable |

## Section Object

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Section name |
| `size_bytes` | int | Section byte size |
| `kind` | string | `"code"`, `"data"`, or `"custom"` |
