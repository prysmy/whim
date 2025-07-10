# Whim - a simple in-memory database for Rust

> [!WARNING]  
> In active development, the API may change frequently.

## Features
- **Entity**: Define entities with fields and types.
- **Table**: Store and manage entities in tables.
- **Indexing**: Create indexes on entities for fast lookups.
- **Searchable**: Fuzzy search capabilities for string fields.
- **Serialization**: With the `bincode` feature, tables can be serialized and deserialized.

Check out the examples in the `examples` directory for usage.

## Future Plans
- **Better Indexing**: Bincode support / more control over indices.
- **Multi-threading**: Improve usage in multi-threaded environments.
- **Foreign keys / Relations**

## TODO before release 0.1:
- https://rust-lang.github.io/api-guidelines/checklist.html
- tests
- documentation
