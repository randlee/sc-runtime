# `sc-config` Requirements

**ID Range:** REQ-CFG-0001 through REQ-CFG-0004; NFR-CFG-0001 through NFR-CFG-0003  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-23  
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for `sc-config`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md). Entries follow the
shared SC requirement and ADR templates: each entry has the sections
Requirement Statement, Rationale and Success Criteria, in that order. Every id
is binding and is never reused.

## Purpose

`sc-config` gives any Rust program its configuration: JSON files with
environment-variable overrides, deserialised into the caller's own serde types.
It is useful on its own, outside any sc-runtime daemon. In a generated project
both the daemon and the CLI use it to load the same config files, which is how
they agree on the endpoint and the instance root.

## ID Ranges

| Range | Area | Scope |
|---|---|---|
| REQ-CFG-0001 through REQ-CFG-0004 | Requirements | - |
| NFR-CFG-0001 through NFR-CFG-0003 | Non-functional requirements | - |

---

## REQ-CFG-0001: `load` merges `default.json` and `local.json`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Scope: the file layer of the `sc-config` crate (`crates/sc-config`). The design fixes the two file names and that `local.json` is gitignored; the merge rules are decided in this document.

Definitions: `load` is the public function `sc_config::load` with the signature `pub fn load<T: serde::de::DeserializeOwned>(app: AppName) -> Result<T, ConfigError>`. `AppName` is the type `sc_config::AppName`, constructed only by `AppName::new` or `TryFrom<&str>`, which return `ConfigError::InvalidName` with stable code `APP.INVALID_NAME` for a name that is empty, longer than 64 bytes, or contains a character other than an ASCII letter, digit, `-` or `_`; `AppName` exposes its text through `as_str` and `AsRef<str>` and does not implement `Deref`. The config directory is the directory `config` resolved against the process's current working directory. The default file is `default.json` in the config directory. The local file is `local.json` in the config directory. A file's value is the `serde_json::Value` that `serde_json::from_slice` returns for the file's bytes. The merged value is `merge(default file's value, local file's value)` when the local file exists, and the default file's value when the local file is absent. `merge(base, overlay)` is the function the merge table defines. The environment overrides of `app` are the environment-variable overrides that [REQ-CFG-0002](requirements.md) defines for the prefix of `app`. The final value is the merged value after the environment overrides of `app` are applied. The missing-default error is `Err(ConfigError::MissingDefault { path })` with `path` set to the default file's path ([REQ-CFG-0004](requirements.md)).

Merge table, decided in this document:

| `base` is | `overlay` is | `merge(base, overlay)` |
|---|---|---|
| JSON object | JSON object | An object holding every key of `base` and every key of `overlay`. A key present in both has the value `merge(base[key], overlay[key])`. |
| anything | string, number, boolean, array or `null` | `overlay`. Arrays are replaced whole; array elements are never merged or appended. `null` replaces the value; it does not delete the key. |
| string, number, boolean, array or `null` | JSON object | `overlay`. |

Precedence, lowest to highest: the default file, the local file, environment variables.

Worked example. Default file:

```json
{
  "daemon": { "host": "127.0.0.1", "port": 7400 },
  "logging": { "level": "info", "targets": ["stdout", "file"] },
  "stores": { "sqlite": { "path": "data/app.db" } }
}
```

Local file:

```json
{
  "daemon": { "port": 9000 },
  "logging": { "level": null, "targets": ["stdout"] },
  "stores": "disabled"
}
```

Merged value:

```json
{
  "daemon": { "host": "127.0.0.1", "port": 9000 },
  "logging": { "level": null, "targets": ["stdout"] },
  "stores": "disabled"
}
```

`daemon.host` survives because objects merge key by key. `logging.targets` is replaced, not concatenated. `logging.level` becomes `null`. `stores` changes from an object to a string because a non-object overlay replaces the value beneath.

**MUST statements:**

- `crates/sc-config/src/lib.rs` MUST contain a `pub use` item that re-exports `load`.
- `load` MUST read the default file.
- When the default file is absent, `load` MUST return the missing-default error.
- When the local file exists, `load` MUST read the local file.
- `load` MUST obtain each file's value by passing the file's bytes to `serde_json::from_slice::<serde_json::Value>`.
- `load` MUST compute the merged value by the rules of the merge table.
- `load` MUST apply the environment overrides of `app` to the merged value.
- The `Ok` value that `load` returns MUST be the value that `serde_json::from_value::<T>` returns for the final value.

**MUST NOT statements:**

- When the local file is absent, `load` MUST NOT return a `ConfigError` whose `path` is the local file's path.
- The value of `app` MUST NOT change which files `load` reads.
- `load` MUST NOT write any file.
- `load` MUST NOT create any file.
- `load` MUST NOT modify any file.

### Rationale

`config/default.json` is checked in and documents every setting.
`config/local.json` is gitignored and holds one machine's differences.
Recursive object merge lets `local.json` change one nested key without
restating its siblings, so a developer's local file stays a few lines long.
Replacing arrays whole avoids the question of how two lists combine, which has
no answer that suits every setting. The design fixes the two file names and
says nothing about merge semantics; this item decides them so that every
project built on `sc-config` layers its files the same way.
### Success Criteria

All tests live in `crates/sc-config` and compare `serde_json::Value`s for
equality, so key order is irrelevant.

1. A unit test writes the `default.json` and `local.json` of the worked example
   into a temporary directory, loads with an empty environment into
   `serde_json::Value`, and asserts the result equals the merged value shown
   above.
2. A unit test with only `default.json` present asserts the load succeeds and
   the result equals the content of `default.json`.
3. A unit test with `default.json` `{"a":[1,2,3]}` and `local.json`
   `{"a":[9]}` asserts the result is `{"a":[9]}`.
4. A unit test with `default.json` `{"a":{"b":1,"c":2}}` and `local.json`
   `{"a":{"b":null}}` asserts the result is `{"a":{"b":null,"c":2}}`.
5. A unit test with `default.json` `{"a":1}` and `local.json` `{"a":{"b":2}}`
   asserts the result is `{"a":{"b":2}}`.
6. A unit test with `default.json` `{"a":1}`, `local.json` `{"a":2}` and an
   environment override that sets `a` to `3` asserts the result is `{"a":3}`.
7. A unit test with no `default.json` in the directory asserts the result is
   `Err` with the missing-default-file variant
   ([REQ-CFG-0004](requirements.md)).
8. A unit test lists the temporary config directory before and after a load
   and asserts the file names and file contents are unchanged.
9. A unit test loads one temporary config directory twice with an empty
   environment, once for app `my-app` and once for app `other-app`, and
   asserts the two results are equal: the app name does not change which
   files are read.

---

## REQ-CFG-0002: Environment overrides named `APP__KEY__KEY`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: The document is the `serde_json::Value` that overrides are applied to. Silent restructuring means changing the document shape implicitly during the override key-path walk instead of applying the stated replacement or error rule. A variable's encoded name is the byte sequence that `OsStr::as_encoded_bytes` returns for the variable's name. The upper-snake form of an app name is the name with every ASCII lower-case letter upper-cased and every `-` replaced by `_`. The prefix is the upper-snake form of the app name followed by `__`. A segment is one piece of the variable name after that prefix, split by `str::split("__")`. The key path `k1, k2, ..., kn` is the sequence of segments converted to ASCII lower case. The key-path walk starts at the document root and moves through keys `k1` to `k(n-1)` in order. The current value is the value the walk is at: the document root at the start, then the value at each key the walk moves through. The target object is the current value after the walk has moved through `k(n-1)`, or the document root when `n` is 1. The override value is the parsed JSON value, or the unchanged value as a JSON string when parsing fails.

This item covers the environment layer of the `sc-config` crate
(`crates/sc-config`). The design fixes only that JSON config files have
environment-variable overrides. The naming scheme, the value parsing and the
path rules below are decided in this document.

Overrides are applied to the `serde_json::Value` produced by merging
`config/default.json` and `config/local.json`
([REQ-CFG-0001](requirements.md)), before that value is deserialised into the
caller's type.

Variable name:

1. App `my-app` gives prefix `MY_APP__`. `_` is already that character, so `my_app` yields the same prefix. `AppName::env_prefix` is the function that implements this rule.
2. The comparison is exact and case-sensitive. Empty segments, application order, and non-Unicode names or values are decided below.
3. `OTHER_APP__DAEMON__PORT` and `MY_APP_DAEMON_PORT` (single underscores) are both ignored for app `my-app`.
5. The segments, in order, are the key path. `MY_APP__DAEMON__MAX_CONNECTIONS` addresses key `max_connections` inside object `daemon`.
6. It is part of the key.
7. A key that contains an ASCII upper-case letter, or that contains `__`,
   cannot be addressed by an override. Array elements cannot be addressed; an
   array is overridden whole.

Value:

9. `8080` is a number, `true` is a boolean, `null` is JSON null, `["a"]` is an array, `{"x":1}` is an object and `"8080"` (with the quotes) is the string `8080`.
10. `debug` becomes `"debug"`, `0.0.0.0` becomes `"0.0.0.0"` and the empty value becomes `""`.

Applying an override with key path `k1, k2, ..., kn`:

11. The walk then moves to the value at that key.
13. An existing value at `kn` is replaced whole, whatever its type. An override value that is an object replaces; it is not merged.

Decided: An application name is `sc_config::AppName`, not a free `&str`.
`AppName::new` accepts only ASCII `A-Z`, `a-z`, `0-9`, `-`, and `_`, length 1
through 64 bytes. Every other string is `ConfigError::InvalidName` with code
`APP.INVALID_NAME`. The only accept and reject list is
`tests/fixtures/app-name-vectors.json`.

Decided: An empty path segment is the cannot-be-applied failure
(`ConfigError::Override`, code `CFG.OVERRIDE`). `MY_APP__`, `MY_APP__A____B`,
and `MY_APP__A__` each return that variant and carry the variable name. The
variable is not ignored.

Decided: Matching variables are applied in ascending order of the
variable name's encoded bytes. Process-environment order is not used. A name
that is a byte-prefix of another is applied first, so
`MY_APP__DAEMON__PORT` wins over `MY_APP__DAEMON` on the keys the longer path
sets.

Decided: A variable whose encoded name starts with the prefix, and whose
name or value is not valid Unicode, returns `ConfigError::NonUnicode` (code
`CFG.NON_UNICODE`) and carries the name as `OsString`. A non-Unicode name
that does not start with the prefix is ignored. The load does not panic and
does not apply a lossy conversion
([NFR-CFG-0001](requirements.md)).

Worked example for app `my-app`. Value after merging the files:

```json
{
  "daemon": { "host": "127.0.0.1", "port": 9000 },
  "logging": { "level": "info", "targets": ["stdout", "file"] }
}
```

Environment:

```text
MY_APP__DAEMON__PORT=8080
MY_APP__DAEMON__HOST=0.0.0.0
MY_APP__DAEMON__TLS=true
MY_APP__DAEMON__MAX_CONNECTIONS=5
MY_APP__LOGGING__LEVEL=debug
MY_APP__LOGGING__TARGETS=["stderr"]
MY_APP__STORES__SQLITE__PATH=data/other.db
MY_APP__LABEL="8080"
OTHER_APP__DAEMON__PORT=1
MY_APP_DAEMON_PORT=2
PATH=/usr/bin
```

Result:

```json
{
  "daemon": {
    "host": "0.0.0.0",
    "port": 8080,
    "tls": true,
    "max_connections": 5
  },
  "logging": { "level": "debug", "targets": ["stderr"] },
  "stores": { "sqlite": { "path": "data/other.db" } },
  "label": "8080"
}
```

`port` is the number `8080` and `label` is the string `"8080"`. `stores` and
`stores.sqlite` did not exist and were created. The last three variables do
not start with `MY_APP__` and are ignored. Adding `MY_APP__DAEMON__PORT__X=1`
to this environment makes the load fail, because `daemon.port` is a number and
cannot be descended into.

**MUST statements:**

- The prefix MUST be the upper-snake form of the app name followed by `__` (two underscores).
- A variable whose encoded name starts with the prefix MUST be treated as an override.
- Every variable whose name does not start with the prefix MUST be ignored, whatever its value.
- The part of the variable name after the prefix MUST be split into segments exactly as Rust `str::split("__")` splits it.
- Each resulting segment MUST be converted to ASCII lower case.
- `crates/sc-config/src/load.rs` MUST parse the variable's value with `serde_json::from_str::<serde_json::Value>`.
- If parsing succeeds, the parsed JSON value MUST be used.
- If parsing fails, the value MUST be used unchanged as a JSON string.
- When the current value is an object that lacks the next key of the key-path walk, an empty object MUST be inserted at that key.
- When the current value of the key-path walk is not an object, the load MUST return `Err(ConfigError::Override { variable })` ([REQ-CFG-0004](requirements.md)).
- In the target object, key `kn` MUST be set to the override value.
- For a key path that both an override and the merged value of `config/default.json` and `config/local.json` set, the loaded value MUST be the override value.

**MUST NOT statements:**

- A single `_` MUST NOT separate segments.
- The document MUST NOT be silently restructured.

### Rationale

Deployments and CI need to change a setting without editing a file, and the
design requires environment-variable overrides for that reason. `__` is the
separator so that a key may itself contain `_`, which snake-case keys do.
Lower-casing the segments lets variables follow the upper-case convention
while JSON keys stay snake case. Parsing the value as JSON lets a number or a
boolean be overridden with the right type, so the caller's `u16` port field
still deserialises; falling back to a string keeps plain values such as paths
and log levels free of quoting. Failing on a path that runs through a
non-object value surfaces a typo or a stale variable instead of reshaping the
document behind the user's back. The design does not state a naming scheme;
this item decides one so that every `sc-config` application is overridden the
same way.
### Success Criteria

All tests live in `crates/sc-config`, use an in-memory environment and a
temporary config directory ([REQ-CFG-0003](requirements.md)), and do not set
process environment variables.

1. A unit test loads the merged value and the environment of the worked
   example into `serde_json::Value` for app `my-app` and asserts the result
   equals the result shown above.
2. A unit test asserts, one case each, that the values `8080`, `true`, `null`,
   `["a"]` and `{"x":1}` produce a JSON number, boolean, null, array and
   object at the addressed key.
3. A unit test asserts that the values `debug`, `0.0.0.0` and the empty value
   produce the JSON strings `"debug"`, `"0.0.0.0"` and `""`, and that the
   value `"8080"` (with quotes) produces the JSON string `"8080"`.
4. A unit test with file value `{"daemon":{"port":1}}` and only
   `OTHER_APP__DAEMON__PORT=2`, `MY_APP_DAEMON_PORT=3` and
   `my_app__daemon__port=4` set asserts that the result for app `my-app` is
   `{"daemon":{"port":1}}`.
5. A unit test with file value `{}` and `MY_APP__A__B__C=1` asserts the result
   is `{"a":{"b":{"c":1}}}`.
6. A unit test with file value `{"daemon":{"port":7400}}` and
   `MY_APP__DAEMON__PORT__X=1` asserts the result is `Err` with the
   cannot-be-applied variant and that the error names the variable
   `MY_APP__DAEMON__PORT__X`.
7. A unit test with file value `{"daemon":{"host":"h","port":1}}` and
   `MY_APP__DAEMON={"port":2}` asserts the result is `{"daemon":{"port":2}}`
   (replaced, not merged).
8. A unit test for app `my-app` deserialises into a struct with a field
   `port: u16` inside `daemon`, with `MY_APP__DAEMON__PORT=8080` set, and
   asserts `port == 8080`.

---

## REQ-CFG-0003: `Loader` takes explicit config directory and environment

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: JSON means JavaScript Object Notation. API means application programming interface. A temporary-directory API is a function that creates a new, uniquely named directory on each call, such as `TempDir::new` in `crates/sc-config/tests/support`. A directory path derives from a temporary-directory API call when it is the directory that call returned or a path beneath that directory. The app name is the `sc_config::AppName` value that names the application, specified in [REQ-CFG-0001](requirements.md). `merge(base, overlay)` recursively merges object keys and replaces other values with the overlay, including arrays and null, as specified in [REQ-CFG-0001](requirements.md). The merged document is the result of `merge` applied to `default.json` and `local.json` read from the config directory, as specified in [REQ-CFG-0001](requirements.md). The environment-override rules are the rules of [REQ-CFG-0002](requirements.md) that select matching app-prefixed variables, decode their key paths and values, and apply them to the merged document. `ConfigError` is the error type of `sc-config`, specified in [REQ-CFG-0004](requirements.md). The default inputs of `load` are the path `config` resolved against the current working directory, passed as `config_dir`, and the pairs returned by `std::env::vars_os`, passed as `environment`. A fixed host path is a literal string that begins with `/tmp/`, `/home/` or `/Users/`. Code under `crates/sc-config` includes `src/` and test code.

This item covers the `Loader` type and the free function `load` of the `sc-config` crate (`crates/sc-config`).

**MUST statements:**

- The crate MUST export a public type `Loader`.
- The return type of `Loader::new` MUST be `Loader`.
- `Loader::new` MUST take the app name as an argument.
- A `Loader` MUST be given the config directory (a filesystem path) explicitly by the caller.
- `Loader::new` MUST require the caller to provide the `environment` argument.
- The `environment` argument MUST have type `Vec<(OsString, OsString)>`.
- The entries of `environment` MUST be iterated as `(variable name, value)` pairs.
- `Loader::load` MUST be a public method.
- `Loader::load` MUST be generic over a type parameter `T`.
- The declaration of `Loader::load` MUST bound `T` by `serde::de::DeserializeOwned` alone.
- The return type of `Loader::load` MUST be `Result<T, ConfigError>`.
- `Loader::load` MUST read `default.json` only from the given config directory.
- `Loader::load` MUST read `local.json` only from the given config directory.
- `Loader::load` MUST take overrides only from the given environment source.
- `Loader::load` MUST merge the values from its supplied configuration directory using the `merge(base, overlay)` rules in [REQ-CFG-0001](requirements.md).
- `Loader::load` MUST apply the environment-override rules to the merged document with the pairs of its `environment` argument.
- `load::<T>(app)` MUST return the same result as `Loader::new(app, config_dir, environment).load::<T>()` called with the default inputs.
- When reading the process environment, `load` MUST use `std::env::vars_os`.
- Every directory path in `sc-config` test code MUST derive from a temporary-directory API call in the same test.

**MUST NOT statements:**

- `Loader` methods MUST NOT call any `std::env` function that reads environment variables.
- The free function `load` MUST NOT use `std::env::vars`.
- Code under `crates/sc-config` MUST NOT call any `std::env` function that changes the process environment.
- Code under `crates/sc-config` MUST NOT call `std::env::set_current_dir`.
- Files under `crates/sc-config` MUST NOT contain a fixed host path.

### Rationale

`std::env::vars` can panic when an environment variable is not valid Unicode. The free function `load` uses `std::env::vars_os`, and environment pairs are `OsString`, so non-Unicode input is reported instead of causing that panic.

The process environment and the current working directory are global to the
process. A test that sets a variable or changes directory races with every
other test in the same binary, and Rust's test runner runs tests in parallel
by default. The repository requires every test that touches the filesystem to
use its own temporary directory and to pass when run in parallel
([NFR-RUN-0005](../requirements.md)). Passing the directory and the
environment in explicitly makes that possible for this crate's own tests and
for the tests of any program that uses it. `std::process::Command::env` and
`std::process::Command::current_dir` change only a child process and remain
allowed.

[NFR-RUN-0005](../requirements.md) bans literal temporary-directory and
home-directory paths in non-test code; this document extends the ban to this
crate's test code. This document also decides the `config` directory and the
process environment as the default inputs of `load`; the design fixes only the
file names `config/default.json` and `config/local.json`.
### Success Criteria

1. Inspection: `crates/sc-config/src/lib.rs` exports `AppName`, `Loader`,
   `load` and `ConfigError`.
2. A unit test creates two temporary directories with different
   `default.json` content, builds one `Loader` per directory, and asserts each
   returns its own directory's values.
3. A unit test passes an in-memory environment containing `MY_APP__A=1` to a
   `Loader` for app `my-app` over file value `{"a":0}`, without setting any
   process variable, and asserts the result is `{"a":1}`.
4. A unit test passes an empty in-memory environment and asserts the result
   equals the file value.
5. An integration test covers the free function `load`. It
   writes `config/default.json` with content `{"a":0}` under a temporary
   directory `D`. It starts a helper executable with `std::process::Command`,
   with `.current_dir(D)`, `.env_clear()` and `.env("MY_APP__A", "1")`. The helper calls
   `sc_config::load::<serde_json::Value>` for `AppName::new("my-app")` and prints the result as
   JSON on standard output. The test asserts the printed value is `{"a":1}`.
   How the helper executable is built (for example a test-only binary target
   of the crate) is left to the implementation.
6. An integration test covers the MUST NOT on `Loader` reading environment variables. It starts a helper executable with a cleared child environment and
   `.env("MY_APP__A", "1")` set on that child. The helper builds a
   `Loader` for app `my-app` over a directory whose `default.json` is
   `{"a":0}`, with an empty in-memory environment, and prints the result. The
   test asserts the printed value is `{"a":0}`: a matching variable in the
   process environment is not read.
7. `cargo test -p sc-config` passes with the default parallel test runner,
   and passes when two such commands run at the same time.
8. `grep -rnE "set_var|remove_var|set_current_dir" crates/sc-config` returns
   no match.
9. `grep -rn "env::vars()" crates/sc-config/src` returns no match.
10. `grep -rnE "/tmp/|/home/|/Users/" crates/sc-config` returns no match: no
    fixed temporary or home path in code or tests.

---

## REQ-CFG-0004: `ConfigError` variants name the file or variable at fault

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: The failure table is the table of five load failures in this requirement. Failure N is row N of the failure table. A failure's data is the set of values named in that row's third column. The five failure variants are `MissingDefault`, `Read`, `Parse`, `Override` and `Deserialize`, for failures 1 to 5 in that order. A trait implementation for `ConfigError` is a `#[derive]` of the trait on `ConfigError` or an `impl` block of the trait for `ConfigError` in `crates/sc-config/src/error.rs`.

This item covers the public error enum `ConfigError` of the `sc-config` crate
(`crates/sc-config`). It is the error type of `load` and of `Loader::load`.

| # | Failure | Data the variant MUST carry |
|---|---|---|
| 1 | `default.json` does not exist in the config directory | the full path that was looked for |
| 2 | `default.json` or `local.json` exists but cannot be read (for example permission denied, or the path is a directory) | the file path |
| 3 | a file's content is not valid JSON; this includes content that is not valid UTF-8, because JSON text is UTF-8 | the file path, and the line and column reported by `serde_json::Error::line()` and `column()` |
| 4 | an environment override cannot be applied because its key path passes through a value that is not a JSON object | the full environment variable name |
| 5 | the merged document does not deserialise into the caller's type `T` | the message of the `serde_json::Error`, the paths of the files that were read, and the names of the environment variables that were applied |

7. Failure 5 lists contributing files and variables because the type mismatch
   is found in the merged document, where the origin of a value is no longer
   known.

Decided: The five load failures are `MissingDefault { path }`,
`Read { path, source }` where `source` is `std::io::Error`,
`Parse { path, line, column, message }`, `Override { variable }`, and
`Deserialize { message, files, variables }`. Name construction adds
`InvalidName { name }` with code `APP.INVALID_NAME`. A prefix-matching
non-Unicode environment entry adds `NonUnicode { variable }`. Stable codes
and recovery steps are the error inventory in
[ADR-CFG-0001](architecture.md).

**MUST statements:**

- `ConfigError` MUST have one distinct variant for each of the five failures of the failure table.
- Each of the five failure variants MUST carry its failure's data as typed fields.
- A `local.json` that exists but cannot be read MUST produce failure 2.
- Failure 1 MUST produce the variant `MissingDefault`.
- Every I/O error on `default.json` or `local.json` other than failure 1 MUST produce the variant `Read`.
- `crates/sc-config/src/error.rs` MUST contain a trait implementation of `std::fmt::Debug` for `ConfigError`.
- `crates/sc-config/src/error.rs` MUST contain a trait implementation of `std::fmt::Display` for `ConfigError`.
- `crates/sc-config/src/error.rs` MUST contain a trait implementation of `std::error::Error` for `ConfigError`.
- The `Display` text of a variant that carries a file path MUST contain that path.
- The `Display` text of a variant that carries a variable name MUST contain that name.
- The `Display` text of the `Parse` variant MUST contain its line number.
- The `Display` text of the `Parse` variant MUST contain its column number.

**SHOULD statements:**

- The failure 2 variant SHOULD also carry the underlying `std::io::Error` or its `std::io::ErrorKind`.

**MUST NOT statements:**

- A `local.json` that exists but cannot be read MUST NOT be skipped.

- A missing `local.json` MUST NOT produce an error.

### Rationale

A configuration error is the first failure a new user meets, and in a
generated project `main.rs` prints this error to the terminal and exits before
any logging exists. "Invalid config" with no file or key leaves the user
searching two files and the environment. Naming the file, the position or the
variable turns the message into a fix. Distinct variants with typed fields let
a calling program branch on the failure, for example to print a hint that
`config/default.json` must be run from the project root, without parsing
message text.
### Success Criteria

All tests live in `crates/sc-config` and use a `Loader` with a temporary
directory and an in-memory environment.

1. A test with an empty config directory asserts `Err` with the failure 1
   variant and that the carried path ends with `default.json`.
2. A test in which `default.json` is a directory (not a file) asserts `Err`
   with the failure 2 variant carrying that path.
3. A test in which `local.json` is a directory asserts `Err` with the failure
   2 variant carrying the `local.json` path.
4. A test with `default.json` content `{"a": 1,\n  "b": }` asserts `Err` with
   the failure 3 variant, the `default.json` path, and line `2`.
5. A test with a valid `default.json` and `local.json` content `{` asserts
   `Err` with the failure 3 variant carrying the `local.json` path. A second
   case with `default.json` containing the bytes `FF FE 00` asserts the
   failure 3 variant carrying the `default.json` path.
6. A test with file value `{"a":1}` and variable `MY_APP__A__B=2` for app
   `my-app` asserts `Err` with the failure 4 variant carrying the name
   `MY_APP__A__B`.
7. A test with file value `{"port":"not-a-number"}`, a `local.json` of `{}`,
   variable `MY_APP__NAME=x`, and target struct `{ port: u16, name: String }`
   asserts `Err` with the failure 5 variant, that the carried file list
   contains both file paths, and that the carried variable list contains
   `MY_APP__NAME`.
8. For each of the five errors produced above, a test asserts that
   `to_string()` contains the carried file path or variable name, and for
   failure 3 the line number.
9. A compile-time test asserts `ConfigError: std::error::Error`, for example
   `fn assert_error<E: std::error::Error>() {}` called with `ConfigError`.

---

## NFR-CFG-0001: Public API returns `Result<T, ConfigError>`; no panics

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Scope: all code under `crates/sc-config/src`. This item applies the panic rule and error rule of [NFR-RUN-0009](../requirements.md) to `sc-config` and adds stricter `sc-config` entries. No commented exception is allowed.

Definitions: A public function is a `pub fn` or `pub` method of `sc-config` callable from outside the crate; implementations of `Display`, `Debug`, `std::error::Error` and `From` on public types are not public functions. The fallible functions are `AppName::new`, `Loader::load`, `load` and any other public function that reads a file, reads the environment or rejects at least one argument value; `Loader::new` is not a fallible function. `ConfigError` is the public typed error enum of `sc-config` ([REQ-CFG-0004](requirements.md)). Test code is code inside a `#[cfg(test)]` module or under `crates/sc-config/tests/`. Non-test code is code in `crates/sc-config/src` that is not test code. Reachable code is non-test code that a public function or a `Display`, `Debug`, `std::error::Error` or `From` implementation on a public type can execute. An indexable value is a slice, string, `Vec` or `serde_json::Value`. Checked access is reading an element of an indexable value with `get`, a pattern match or an iterator. An opaque error type is `Box<dyn Error>` or `anyhow::Error`. An output macro is `print!`, `println!`, `eprint!`, `eprintln!`, `dbg!` or a logging macro. A panicking construct is any of `.unwrap()`, `.expect(`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, `assert!`, `assert_eq!`, `assert_ne!` or a `[` index expression on an indexable value. Any input means any app name, any directory path, any file content and any environment content, including environment variables that are not valid Unicode.

Decided in this document, stricter than NFR-RUN-0009: `assert!`, `assert_eq!` and `assert_ne!` are panicking constructs, and calling `std::env::vars()` is banned.

**MUST statements:**

- Every fallible function MUST have the return type `Result<T, ConfigError>` for some type `T`.
- `Loader::new` MUST have the return type `Loader`.
- `AppName::new` MUST have the return type `Result<AppName, ConfigError>`.
- Reachable code MUST read every element of an indexable value by checked access.
- Reachable code that reads the process environment MUST call `std::env::vars_os()`.
- When called with any input, a public function MUST return without panicking.

**MUST NOT statements:**

- A public function MUST NOT have a return type of `Option` for reporting failure.
- A public function MUST NOT have a return type containing an opaque error type.
- Reachable code MUST NOT call an output macro.
- Reachable code MUST NOT use a panicking construct.
- Non-test code MUST NOT call `std::env::vars()`.

### Rationale

The design states this directly for `sc-config`: every method returns a
discriminated union and nothing panics. Configuration is loaded at the top of
`main`, before logging or any error reporting exists. A panic at that point
prints a Rust backtrace to a user whose actual mistake is a typo in a JSON
file, and an agent driving the program gets output it cannot parse. A typed
error lets the generated `main.rs` print one clear line and return an exit
code.
### Success Criteria

1. `grep -rnE
   "\.unwrap\(\)|\.expect\(|panic!|unreachable!|todo!|unimplemented!|assert(_eq|_ne)?!"
   crates/sc-config/src` prints no line outside `#[cfg(test)]` modules, or
   each remaining match is shown by review to be unreachable from a public
   function. A comment on the line is not grounds for accepting a match.
2. Inspection of every `pub fn` in `crates/sc-config/src` that can fail: the
   return type is `Result<_, ConfigError>`. Standard trait implementations
   are outside this check. `Loader::new` cannot fail and returns `Loader`.
   `AppName::new` can fail and returns `Result<AppName, ConfigError>`.
3. Inspection of code reachable from a public function in
   `crates/sc-config/src` finds no `[` index expression on a slice, string,
   `Vec` or `serde_json::Value`.
4. `grep -rn "env::vars()" crates/sc-config/src` returns no match.
5. A test feeds each of these to a `Loader` and asserts the call returns
   `Err(ConfigError)` and does not panic: an empty `default.json`; a
   `default.json` containing the bytes `FF FE 00`; a `default.json` truncated
   mid-object (`{"a":`); a config directory path that does not exist.
6. A test feeds a `default.json` whose top-level value is an array (`[1,2]`)
   plus override `MY_APP__A=1` for app `my-app` and asserts `Err`, no panic.
7. A test feeds override values `{`, `]`, a 1 MB string of `[` characters, and
   the empty value, and asserts each call returns (`Ok` or `Err`) without
   panicking.
8. `cargo test -p sc-config` reports no test that fails with "panicked".

---

## NFR-CFG-0002: `sc-config` Phase D dependency footprint

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: The sc-config manifest is the file `crates/sc-config/Cargo.toml`. A dependency entry is one key of a dependency table in the sc-config manifest. The sc-config dependency tree is the output of `cargo tree -p sc-config -e normal --prefix none`. A hand-written impl block is an `impl` block whose full source text appears in the file, rather than one that a derive or other macro generates.

This record binds the sc-config manifest of the Phase D deliverable and owns its third-party dependency list. A later async or reload option is not forbidden and creates no work in Phase D (PRD-ENG-5/6). [REQ-RUN-0005](../requirements.md) owns the edges between the crates of this workspace, the forbidden edges and the boundary manifest under `boundaries/sc-config/`, and refers to this record for the third-party list. For Phase D, the forbidden edges of `sc-config` are `sc-observability`, `sc-observability-otlp` (the OTel export crate of `sc-observability`), `tokio`, `sc-transport`, `sc-command` and `sc-runtime`; [REQ-RUN-0005](../requirements.md) states the obligation and the `forbidden_edges` entries that `sc-lint-boundary` checks, and this record adds only the `cargo tree` check in criterion 3. Tests create a private directory under `std::env::temp_dir` and remove the directory when the test ends.

**MUST statements:**

- The set of keys in the `[dependencies]` table of the sc-config manifest MUST equal the set {`serde`, `serde_json`}.
- The file `crates/sc-config/src/error.rs` MUST contain a hand-written `impl std::fmt::Display for ConfigError` block.
- The file `crates/sc-config/src/error.rs` MUST contain a hand-written `impl std::error::Error for ConfigError` block.

**MUST NOT statements:**

- The sc-config manifest MUST NOT contain a `[dev-dependencies]` table with any key.
- The sc-config manifest MUST NOT contain a dependency entry with `optional = true`.
- The sc-config manifest MUST NOT contain a `[target.*.dependencies]` table.
- The sc-config dependency tree MUST NOT contain a line that starts with `thiserror `.
- The sc-config dependency tree MUST NOT contain a line that starts with `anyhow `.

### Rationale

The design lists exactly `serde` and `serde_json` as the dependencies of
`sc-config` and states that `sc-observability` and its OTel export crate stay
independent of `sc-config`.
`sc-config` is the first thing a program calls and is linked into every CLI as
well as every daemon. A configuration crate that pulled in an async runtime or
a logging stack would add compile time and binary size to every small tool
that only wants to read a JSON file, and would make initialisation order
circular: logging is configured from the values this crate loads.
### Success Criteria

1. For Phase D, inspection of `crates/sc-config/Cargo.toml`: the `[dependencies]` table has
   the keys `serde` and `serde_json` and no others, no entry has
   `optional = true`, `[dev-dependencies]` has no key, and the file has no
   `[target.*.dependencies]` table.
2. For Phase D, `cargo tree -p sc-config -e normal --depth 1` lists `serde` and
   `serde_json` as the only direct dependencies.
3. For Phase D, `cargo tree -p sc-config -e normal --prefix none` prints no line beginning
   with `<name> ` (the crate name followed by a space) for each of these
   names: `sc-observability`, `sc-observability-otlp`, `tokio`,
   `sc-transport`, `sc-command`, `sc-runtime`, `thiserror`, `anyhow`. The
   match is on the start of the line because the first line of the output
   holds the checkout path, which may itself contain `sc-runtime`.
4. The boundary manifest under `boundaries/sc-config/` and the
   `sc-lint-boundary` run are checked by the criteria of
   [REQ-RUN-0005](../requirements.md).

---

# Requirement: `sc-config` load calls are synchronous in Phase D

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## NFR-CFG-0003: `sc-config` load calls are synchronous in Phase D

**Status:** Active  

### Requirement Statement

`sc-config` provides synchronous start-up configuration. The Phase D deliverable is built
with no file watcher, no reload and no change notification. Async
configuration and reload are deferred, not forbidden: nothing in this record
stops a consumer or a later version from adding them, including an async
API next to the synchronous one
([PRD-ENG-5, section 9](../prd/sc-runtime-prd.md)).

This item binds `load` and `Loader::load` in `crates/sc-config/src` for the Phase D deliverable.

Definitions: A load call is one call to `load` or `Loader::load` in `crates/sc-config/src`. Load work is the file reads, override application and deserialisation that a load call performs. The calling thread is the thread that makes a load call. An async runtime is an executor, such as Tokio, that polls futures. A configuration file is `default.json` or `local.json` in the configuration directory passed to the load call. A configuration source is a configuration file or an environment variable that a load call reads.

**MUST statements:**

- A load call made on a thread with no async runtime running MUST return `Ok` for a configuration directory holding a valid `default.json`.
- The load work of a load call in `crates/sc-config/src` MUST run on the calling thread.
- When a configuration source changes after a load call returns, the value returned by the load call MUST keep the contents it had at return.

**MUST NOT statements:**

- A load call MUST NOT spawn a thread.
- A load call MUST NOT spawn an async task.

### Rationale

Every current user reads its configuration once at start-up, so Phase D builds
only that. A synchronous function can be called from `main` before a runtime
exists and from inside an async `main` alike. A changed file is seen only
through a new explicit load call. The owner wants async
configuration and reload kept open as options, possibly later as an async
trait extending the synchronous one; under PRD-ENG-5 and PRD-ENG-6 this record
neither forbids them nor binds a mechanism for them.

### Success Criteria

1. Inspection of the signatures of `load` and `Loader::load` in
   `crates/sc-config/src` finds that neither is an `async fn` and that each
   returns `Result<T, ConfigError>`, not a `Future`.
2. Inspection of `load`, `Loader::load` and every function they call in
   `crates/sc-config/src` finds no `thread::spawn`, `thread::Builder`,
   `tokio::spawn` or other task spawn.
3. A test calls `Loader::load` from a plain `#[test]` function with no async
   runtime and asserts it returns `Ok`.
4. A test loads, then rewrites `default.json` with a different value, and
   asserts the value returned by the first load is unchanged and that a second
   explicit load returns the new value.
