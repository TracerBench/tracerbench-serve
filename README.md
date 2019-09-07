# tracerbench-serve

Serves recorded response sets for benchmarking.

## recorded response set

Serde deserialize format:

Recorded response sets
Seq (len 6)
  Body table
  Header name table
  Header value table
  Headers table
  Response table
  Recorded response set table

Body table:
Seq of
  Bytes

Header name table:
Seq of
  String

Header value table:
Seq of
  String

Headers table
Seq of
  Seq of
    (
      usize, // name table index
      usize, // value table index
    )

Response table
Seq of
  (
    u16, // status
    usize, // headers table index
    Option<usize>, // body table index
  )

Response set table
Seq of
  Map
    socksPort: u16,
    name: String,
    entryKey: String,
    requestKeyProgram: Request key,
    requestKeyMap:
      Map String: usize // key to response_index

Request key
Seq
  literals
  Bytes

literals
  Map
    type: String,
    content: Value

