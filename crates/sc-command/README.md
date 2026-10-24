# sc-command

Lossless response envelope for a JSON-first CLI or API, plus the conversions
that put that envelope on an Axum response or an rmcp tool result.

The default build is the envelope and `OpError`. The `server` feature adds
the Axum and rmcp conversions and is off by default.

```rust
use sc_command::{Envelope, ErrorKind, OpError};
use sc_observability_types::{ErrorCode, Remediation};

let error = OpError::new(
    ErrorKind::Unavailable,
    ErrorCode::new_static("DAEMON.NOT_RUNNING"),
    "the daemon is not running",
)
.with_suggestion("run my-app daemon start");
let envelope = Envelope::from(Err::<(), _>(error));
let value = serde_json::to_value(&envelope)?;
assert_eq!(value["ok"], false);
assert_eq!(value["data"], serde_json::Value::Null);
assert_eq!(
    value["error"]["suggested_action"],
    "run my-app daemon start"
);
let again: Envelope<()> = serde_json::from_value(value)?;
assert!(!again.is_ok());
# Ok::<(), serde_json::Error>(())
```

`with_suggestion` stores `sc_observability_types::Remediation`. The wire field
`suggested_action` is that remediation as a JSON string, or JSON `null` when
no remediation is set.
