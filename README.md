# tracerbench-serve

Serves recorded response sets for benchmarking. The goal is to serve recorded responses with low variance in memory and CPU cost.

## For Guide, API Reference & Contributing Info

https://www.tracerbench.com/

## Building

Follow the instructions at https://rustup.rs.

```sh
rustup toolchain install stable
```

```sh
cargo build --release
```

## recorded response set

Serde deserialize format:

### Recorded response sets:

- Seq length 6
  - Body table
  - Header name table
  - Header value table
  - Headers table
  - Response table
  - Recorded response set table

### Body table:

- Seq of
  - Bytes

### Header name table:

- Seq of
  - String

### Header value table:

- Seq of
  - String

### Headers table

- Seq of
  - Seq of
    - Seq length 2
      - usize ( name table index )
      - usize ( value table index )

### Response table

- Seq of
  - Seq length 3
    - u16 ( status )
    - usize ( headers table index )
    - Option<usize> ( body table index )

### Response set table

- Seq of
  - Map
    - socksPort:
      - u16
    - name:
      - String
    - entryKey:
      - String
    - requestKeyProgram:
      - Request key
    - requestKeyMap:
      - Map
        - String: ( request key )
          - usize ( response_index )

### Request key

- Seq length 2
  program literals
  Bytes (program bytecode)

#### program literals

- Map
  - type:
    - String
  - content:
    - Value
