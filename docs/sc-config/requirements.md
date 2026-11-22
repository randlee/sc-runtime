# `sc-config` Requirements

**ID Range:** REQ-CFG-0001 through REQ-CFG-0010; NFR-CFG-0001 through NFR-CFG-0004  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-22  
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for `sc-config`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md). Entries follow the
shared SC requirement and ADR templates: each entry has the sections
Requirement Statement, Rationale and Success Criteria, in that order. An id is
never reused. Only ids with **Status: Active** are binding. An id with
**Status: Proposed** is a design under review: it binds nothing, sprint plans
must not cite it as governing, and it becomes binding only when a later change
sets its status to Active.

## Purpose

`sc-config` gives any Rust program its configuration: JSON files with
environment-variable overrides, deserialised into the caller's own serde types.
It is useful on its own, outside any sc-runtime daemon. In a generated project
both the daemon and the CLI use it to load the same config files, which is how
they agree on the endpoint and the instance root.

## ID Ranges

| Range | Area | Status |
|---|---|---|
| REQ-CFG-0001 through REQ-CFG-0004 | Requirements | Active |
| REQ-CFG-0005 through REQ-CFG-0009 | Requirements: layered engine ([ADR-CFG-0004](architecture.md)) | Proposed |
| REQ-CFG-0010 | Requirements: standard layer order ([ADR-CFG-0005](architecture.md)) | Proposed |
| NFR-CFG-0001 through NFR-CFG-0003 | Non-functional requirements | Active |
| NFR-CFG-0004 | Non-functional requirements | Proposed |

---

## REQ-CFG-0001: `load` merges `default.json` and `local.json`

**Status:** Active  

**Proposed amendment:** [ADR-CFG-0004](architecture.md) and [REQ-CFG-0008](requirements.md) re-express this behaviour on the layered engine without changing it. Until they are Active this item stands as written.

### Requirement Statement

This item covers the file layer of the `sc-config` crate (`crates/sc-config`).
The design fixes the two file names and that `local.json` is gitignored. The
merge rules below are decided in this document.

The crate MUST export this function:

```rust
pub fn load<T: serde::de::DeserializeOwned>(app: AppName) -> Result<T, ConfigError>
```

`app` is an `sc_config::AppName`. Callers construct it with `AppName::new` or
`TryFrom<&str>`. A name that is empty, longer than 64 bytes, or outside ASCII
letters, digits, `-`, and `_` fails in that constructor with
`ConfigError::InvalidName` and stable code `APP.INVALID_NAME`. It does not
reach `load`. The name still selects only the environment-variable prefix
(entry 9). Display is `as_str` or `AsRef<str>`. The type does not implement
`Deref`.

1. `load` MUST read `config/default.json`, resolved against the process's
   current working directory.
2. `config/default.json` MUST exist. If it does not, `load` MUST return the
   `ConfigError` variant for a missing default file
   ([REQ-CFG-0004](requirements.md)).
3. `load` MUST read `config/local.json` from the same directory when that file
   exists.
4. An absent `config/local.json` MUST NOT be an error. The result is then
   built from `config/default.json` alone.
5. Each file MUST be read as bytes and parsed as one JSON document into a
   `serde_json::Value` (`serde_json::from_slice`).
6. `load` MUST merge the `local.json` value over the `default.json` value with
   the function `merge(base, overlay)` defined in the table below.
7. `load` MUST then apply the environment-variable overrides of app `app` to
   the merged value. Those overrides take precedence over both files
   ([REQ-CFG-0002](requirements.md)).
8. `load` MUST deserialise the final value into `T` exactly once, with
   `serde_json::from_value::<T>`.
9. The `app` argument MUST NOT change which files are read. It selects only
   the environment-variable prefix.
10. `load` MUST NOT write, create or modify any file.

`merge(base, overlay)`, decided in this document:

| `base` is | `overlay` is | Result |
|---|---|---|
| JSON object | JSON object | An object holding every key of `base` and every key of `overlay`. A key present in both has the value `merge(base[key], overlay[key])`. |
| anything | string, number, boolean, array or `null` | `overlay`. Arrays are replaced whole; array elements are never merged or appended. `null` replaces the value; it does not delete the key. |
| string, number, boolean, array or `null` | JSON object | `overlay`. |

Precedence, lowest to highest: `config/default.json`, `config/local.json`,
environment variables.

Worked example. `config/default.json`:

```json
{
  "daemon": { "host": "127.0.0.1", "port": 7400 },
  "logging": { "level": "info", "targets": ["stdout", "file"] },
  "stores": { "sqlite": { "path": "data/app.db" } }
}
```

`config/local.json`:

```json
{
  "daemon": { "port": 9000 },
  "logging": { "level": null, "targets": ["stdout"] },
  "stores": "disabled"
}
```

Merged value, with no environment overrides set:

```json
{
  "daemon": { "host": "127.0.0.1", "port": 9000 },
  "logging": { "level": null, "targets": ["stdout"] },
  "stores": "disabled"
}
```

`daemon.host` survives because objects merge key by key. `logging.targets` is
replaced, not concatenated. `logging.level` becomes `null`. `stores` changes
from an object to a string because a non-object overlay replaces whatever is
beneath it.

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
   files are read (entry 9).

---

## REQ-CFG-0002: Environment overrides named `APP__KEY__KEY`

**Status:** Active  

**Proposed amendment:** [ADR-CFG-0004](architecture.md) and [REQ-CFG-0008](requirements.md) re-express this behaviour on the layered engine without changing it. Until they are Active this item stands as written.

### Requirement Statement

This item covers the environment layer of the `sc-config` crate
(`crates/sc-config`). The design fixes only that JSON config files have
environment-variable overrides. The naming scheme, the value parsing and the
path rules below are decided in this document.

Overrides are applied to the `serde_json::Value` produced by merging
`config/default.json` and `config/local.json`
([REQ-CFG-0001](requirements.md)), before that value is deserialised into the
caller's type.

Variable name:

1. The prefix MUST be the app name with every ASCII lower-case letter
   upper-cased and every `-` replaced by `_`, followed by `__` (two
   underscores). App `my-app` gives prefix `MY_APP__`. `_` is already that
   character, so `my_app` yields the same prefix. `AppName::env_prefix` is
   the function that implements this rule.
2. A variable MUST be treated as an override if and only if its encoded name
   starts with that prefix. The comparison is exact and case-sensitive. Empty
   segments, application order, and non-Unicode names or values are decided
   below.
3. Every variable whose name does not start with the prefix MUST be ignored,
   whatever its value. `OTHER_APP__DAEMON__PORT` and `MY_APP_DAEMON_PORT`
   (single underscores) are both ignored for app `my-app`.
4. The part of the name after the prefix MUST be split at every
   non-overlapping occurrence of `__`, scanning left to right (Rust
   `str::split("__")`).
5. Each resulting segment MUST be converted to ASCII lower case. The segments,
   in order, are the key path. `MY_APP__DAEMON__MAX_CONNECTIONS` addresses key
   `max_connections` inside object `daemon`.
6. A single `_` MUST NOT separate segments. It is part of the key.
7. A key that contains an ASCII upper-case letter, or that contains `__`,
   cannot be addressed by an override. Array elements cannot be addressed; an
   array is overridden whole.

Value:

8. The variable's value MUST be parsed with
   `serde_json::from_str::<serde_json::Value>`.
9. If parsing succeeds, the parsed JSON value MUST be used. `8080` is a number,
   `true` is a boolean, `null` is JSON null, `["a"]` is an array, `{"x":1}` is
   an object and `"8080"` (with the quotes) is the string `8080`.
10. If parsing fails, the value MUST be used unchanged as a JSON string.
    `debug` becomes `"debug"`, `0.0.0.0` becomes `"0.0.0.0"` and the empty
    value becomes `""`.

Applying an override with key path `k1, k2, ..., kn`:

11. Starting at the document root, for each of `k1` to `k(n-1)`: if the current
    value is an object that lacks the key, an empty object MUST be created at
    that key. The walk then moves to the value at that key.
12. If the current value at any step of that walk, including the root, is not
    an object (it is a string, number, boolean, array or `null`), the load MUST
    fail with the `ConfigError` variant for an override that cannot be applied
    ([REQ-CFG-0004](requirements.md)). The document MUST NOT be silently
    restructured.
13. At the last object, key `kn` MUST be set to the override value. An existing
    value at `kn` is replaced whole, whatever its type. An override value that
    is an object replaces; it is not merged.
14. Overrides MUST take precedence over `config/default.json` and
    `config/local.json`.

**Decided:** An application name is `sc_config::AppName`, not a free `&str`.
`AppName::new` accepts only ASCII `A-Z`, `a-z`, `0-9`, `-`, and `_`, length 1
through 64 bytes. Every other string is `ConfigError::InvalidName` with code
`APP.INVALID_NAME`. The only accept and reject list is
`tests/fixtures/app-name-vectors.json`.

**Decided:** An empty path segment is the cannot-be-applied failure
(`ConfigError::Override`, code `CFG.OVERRIDE`). `MY_APP__`, `MY_APP__A____B`,
and `MY_APP__A__` each return that variant and carry the variable name. The
variable is not ignored.

**Decided:** Matching variables are applied in ascending order of the
variable name's encoded bytes. Process-environment order is not used. A name
that is a byte-prefix of another is applied first, so
`MY_APP__DAEMON__PORT` wins over `MY_APP__DAEMON` on the keys the longer path
sets.

**Decided:** A variable whose encoded name starts with the prefix, and whose
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

### Requirement Statement

This item covers the `Loader` type of the `sc-config` crate
(`crates/sc-config`).

1. The crate MUST export a public type `Loader`.
2. A `Loader` MUST be given three inputs explicitly by the caller: the app
   name, the config directory (a filesystem path), and the environment source.
3. The environment source MUST be an iterator of (variable name, value) pairs
   supplied by the caller. It may be built in memory, for example from a
   `Vec` of pairs.
4. `Loader` MUST expose `Loader::load::<T>()` for any
   `T: serde::de::DeserializeOwned`, returning `Result<T, ConfigError>`.
5. `Loader::load` MUST read `default.json` and `local.json` from the given
   config directory and from nowhere else.
6. `Loader::load` MUST take overrides from the given environment source and
   from nowhere else. It MUST NOT read the process environment.
7. `Loader::load` MUST apply the same file merge
   ([REQ-CFG-0001](requirements.md)) and the same override rules
   ([REQ-CFG-0002](requirements.md)) as the free function `load`.
8. The free function `load::<T>(app)` MUST be the convenience form. It MUST
   behave exactly as a `Loader` given app name `app`, the directory `config`
   resolved against the current working directory, and the process
   environment.
9. When reading the process environment, `load` MUST use
   `std::env::vars_os`. It MUST NOT use `std::env::vars`, which panics when
   any variable is not valid Unicode.
10. No code in the crate may call `std::env::set_var`,
    `std::env::remove_var` or `std::env::set_current_dir`, in `src/` or in
    tests. Setting the environment or the working directory of a child
    process with `std::process::Command::env` and
    `std::process::Command::current_dir` is allowed, because it does not
    change the state of the test process.
11. No file under `crates/sc-config`, code or tests, may contain a literal
    temporary-directory path or a literal home-directory path (`/tmp/`,
    `/home/`, `/Users/`). Tests MUST obtain their directories from a
    temporary-directory API. The repository-wide rule is
    [NFR-RUN-0005](../requirements.md): non-test code contains no such
    literal, and every test that writes files uses its own temporary
    directory. Extending the literal-path ban to this crate's test code is
    decided in this document.

Using `./config` and the process environment as the defaults of `load` is
decided in this document; the design fixes only the file names
`config/default.json` and `config/local.json`. The call takes
`AppName::new("my-app")`.

**Decided:** `Loader::new(app: AppName, config_dir: PathBuf, environment: Vec<(OsString, OsString)>) -> Loader`.
The constructor stores those arguments and does not return `Result`.
`Loader::load` and `load` return `Result<T, ConfigError>`. Environment pairs
are `OsString` so a non-Unicode value is reported instead of panicking.

### Rationale

The process environment and the current working directory are global to the
process. A test that sets a variable or changes directory races with every
other test in the same binary, and Rust's test runner runs tests in parallel
by default. The repository requires every test that touches the filesystem to
use its own temporary directory and to pass when run in parallel
([NFR-RUN-0005](../requirements.md)). Passing the directory and the
environment in explicitly makes that possible for this crate's own tests and
for the tests of any program that uses it.

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
5. An integration test covers the free function `load` (entries 8 and 9). It
   writes `config/default.json` with content `{"a":0}` under a temporary
   directory `D`. It starts a helper executable with `std::process::Command`,
   with `.current_dir(D)` and `.env("MY_APP__A", "1")`. The helper calls
   `sc_config::load::<serde_json::Value>` for `AppName::new("my-app")` and prints the result as
   JSON on standard output. The test asserts the printed value is `{"a":1}`.
   How the helper executable is built (for example a test-only binary target
   of the crate) is left to the implementation.
6. An integration test covers entry 6. It starts a helper executable the same
   way, with `.env("MY_APP__A", "1")` set on the child. The helper builds a
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
    fixed temporary or home path in code or tests (entry 11).

---

## REQ-CFG-0004: `ConfigError` variants name the file or variable at fault

**Status:** Active  

**Proposed amendment:** [REQ-CFG-0009](requirements.md) adds variants and makes `ConfigError` `#[non_exhaustive]`; `MissingDefault` is unchanged. Until it is Active this item stands as written.

### Requirement Statement

This item covers the public error enum `ConfigError` of the `sc-config` crate
(`crates/sc-config`). It is the error type of `load` and of `Loader::load`.

`ConfigError` MUST have one distinct variant for each failure in this table.
Each variant MUST carry the listed data as typed fields, not only inside a
message string.

| # | Failure | Data the variant MUST carry |
|---|---|---|
| 1 | `default.json` does not exist in the config directory | the full path that was looked for |
| 2 | `default.json` or `local.json` exists but cannot be read (for example permission denied, or the path is a directory) | the file path |
| 3 | a file's content is not valid JSON; this includes content that is not valid UTF-8, because JSON text is UTF-8 | the file path, and the line and column reported by `serde_json::Error::line()` and `column()` |
| 4 | an environment override cannot be applied because its key path passes through a value that is not a JSON object | the full environment variable name |
| 5 | the merged document does not deserialise into the caller's type `T` | the message of the `serde_json::Error`, the paths of the files that were read, and the names of the environment variables that were applied |

1. A missing `local.json` MUST NOT produce an error.
2. A `local.json` that exists but cannot be read MUST produce failure 2, not
   be skipped.
3. Failure 1 and failure 2 MUST be distinct variants: "file not found" for
   `default.json` is failure 1, every other I/O error is failure 2.
   The failure 2 variant SHOULD also carry the underlying `std::io::Error` or
   its `std::io::ErrorKind`.
4. `ConfigError` MUST implement `std::fmt::Debug`, `std::fmt::Display` and
   `std::error::Error`.
5. The `Display` text of every variant MUST contain the file path or the
   variable name that the variant carries.
6. The `Display` text of failure 3 MUST contain the line and the column.
7. Failure 5 lists contributing files and variables because the type mismatch
   is found in the merged document, where the origin of a value is no longer
   known.

**Decided:** The five load failures are `MissingDefault { path }`,
`Read { path, source }` where `source` is `std::io::Error`,
`Parse { path, line, column, message }`, `Override { variable }`, and
`Deserialize { message, files, variables }`. Name construction adds
`InvalidName { name }` with code `APP.INVALID_NAME`. A prefix-matching
non-Unicode environment entry adds `NonUnicode { variable }`. Stable codes
and recovery steps are the error inventory in
[ADR-CFG-0001](architecture.md).

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

## REQ-CFG-0005: Layered loading over a caller-ordered list of sources

**Status:** Proposed  

### Requirement Statement

This item covers the layered loading engine of the `sc-config` crate
(`crates/sc-config`). It is the one load pipeline of the crate: `Loader`
([REQ-CFG-0008](requirements.md)) and every consumer-defined stack run on it.
The decision that the caller, not the crate, defines the layers is
[ADR-CFG-0004](architecture.md). The `merge(base, overlay)` function is the
one defined in [REQ-CFG-0001](requirements.md); that table is its only
definition and this item refers to it.

**The trait**

1. The crate MUST export this public trait:

   ```rust
   pub trait LayerSource: Send + Sync {
       /// Provenance of every value this source contributes, unless the
       /// returned `Layer` attributes a leaf to a more specific `Source`.
       fn source(&self) -> Source;
       /// `base` is the value accumulated from every lower layer, read-only.
       /// `Ok(None)` means "absent, contributes nothing" (for example an
       /// optional file that does not exist). It is not an error.
       fn load(&self, base: &serde_json::Value) -> Result<Option<Layer>, ConfigError>;
   }
   ```

2. The trait MUST be object safe and MUST NOT be sealed, so a crate outside
   `sc-config` can implement it for its own source (a keychain, a remote
   store, a CLI profile). `Send + Sync` is a supertrait so that a `Layers`
   can be built in one thread and loaded in another, including inside an
   async runtime.
3. A method added to `LayerSource` after this item becomes Active MUST have a
   default body, so existing implementers keep compiling
   ([NFR-CFG-0004](requirements.md) item 6).

**The layer**

4. `Layer` MUST be a public `#[non_exhaustive]` struct built only through:
   - `Layer::new(value: serde_json::Value) -> Layer`;
   - `Layer::replace_at(self, path: KeyPath) -> Layer`: at `path` the layer's
     value replaces the base's value whole instead of being merged into it;
   - `Layer::attribute(self, path: KeyPath, source: Source) -> Layer`: leaves
     at or under `path` are attributed to `source` in provenance
     ([REQ-CFG-0007](requirements.md)).
5. A layer value need not be a JSON object. A non-object layer value replaces
   everything below it, exactly as `merge` defines for a non-object overlay.
   This keeps [REQ-CFG-0001](requirements.md) behaviour for a `default.json`
   whose top level is not an object.

**The fold**

6. The crate MUST export a builder `Layers`, `Send + Sync`, holding an ordered
   `Vec<Box<dyn LayerSource>>`. `Layers::new() -> Layers` creates an empty
   stack. `Layers::push(self, source: impl LayerSource + 'static) -> Layers`
   appends a source as the new highest-precedence layer. Precedence is exactly
   the push order: a later source overrides an earlier one.
7. `sc-config` MUST NOT impose, reorder or insert layers. No source kind has a
   built-in rank.
8. `Layers::resolve(&self) -> Result<Resolved, ConfigError>` MUST:
   1. start from `base = {}` (an empty JSON object);
   2. call `load(&base)` on every source once, lowest precedence first;
   3. skip a source that returns `Ok(None)`;
   4. stop and return the first `Err`, unchanged;
   5. otherwise set `base` to `fold(base, layer)`, where `fold` is
      `merge(base, layer.value)` except that at every `replace_at` path the
      result holds the layer's value at that path whole;
   6. return `Resolved { value: base, provenance }`, a public
      `#[non_exhaustive]` struct with accessors `value()` and `provenance()`.
9. `Layers::load::<T: DeserializeOwned>(&self) -> Result<T, ConfigError>`
   MUST be `resolve()` followed by exactly one
   `serde_json::from_value::<T>` of the resolved value.
   `Layers::load_with_provenance::<T>(&self) -> Result<Layered<T>, ConfigError>`
   MUST be `resolve()` followed by the same single call, returning
   `Layered<T>` (`#[non_exhaustive]`, accessors `value()`, `provenance()`,
   `into_parts()`). There is no second fold.
10. A deserialisation failure MUST be `ConfigError::Deserialize`
    ([REQ-CFG-0009](requirements.md) item 6). A caller that needs provenance
    for a failing document calls `resolve()` and deserialises itself.
11. An empty stack, or a stack whose sources all return `Ok(None)`, MUST
    resolve to `{}`. It is not an error.
12. The engine MUST NOT read the process environment, the current directory
    or any file itself. Only sources do I/O.
13. `merge` is deliberately not pluggable. Provenance, `replace_at` and the
    guarantees of [REQ-CFG-0001](requirements.md) are defined against this
    one merge; a consumer that needs different combination rules builds the
    combined value in its own `LayerSource`.

### Rationale

PR #153 added a second loader, beside `Loader`, with its own file checks, its
own environment ordering and a fixed flags > environment > files precedence.
Two pipelines drift, and a fixed precedence serves one consumer. A single
engine over caller-ordered sources lets each consumer state its own order
(sc-agent's flags over environment over files, a daemon's files over
environment, a test's in-memory values) without `sc-config` knowing any of
them. Passing the accumulated base to each source lets an environment source
keep the exact path rules of [REQ-CFG-0002](requirements.md) (failing through
a scalar, replacing an object whole) while the engine still folds with one
`merge`. `resolve()` gives one fold that `load` and `load_with_provenance`
share.

### Success Criteria

Tests live in `crates/sc-config`. Sources in tests are in-memory
`ValueSource`s ([REQ-CFG-0006](requirements.md)) unless stated. Counting
tests use a per-test counter (a `thread_local!` `Cell` reset at the start of
the test, or state owned by the test's own source); no test uses a process
static, so results do not depend on test parallelism.

1. Stack `[{"a":1,"b":1}, {"a":2}]` resolves to `{"a":2,"b":1}`; the same
   two sources pushed in reverse order resolve to `{"a":1,"b":1}`.
2. An integration test under `crates/sc-config/tests/` defines a struct that
   implements `LayerSource` (returning `{"x":"custom"}`), pushes it, and
   asserts the load yields `{"x":"custom"}`.
3. A test source records the `base` it receives; for stack
   `[{"a":1}, recorder]` it receives `{"a":1}`.
4. A source returning `Ok(None)` between two value sources leaves the result
   equal to the stack without it.
5. A source returning `Err(e)` makes `resolve` return that same variant; a
   counting source pushed after it records zero calls.
6. A layer `Layer::new({"a":{"x":1}}).replace_at(a)` over base
   `{"a":{"y":2}}` gives `{"a":{"x":1}}`; without `replace_at` it gives
   `{"a":{"x":1,"y":2}}`.
7. Stack `[{"a":1}, 5]` resolves to `5`.
8. An empty stack loads `#[derive(Deserialize, Default)] #[serde(default)]
   struct C { a: u8 }` as `C { a: 0 }`.
9. A `T` whose `Deserialize` increments a per-test counter is deserialised
   once by `load` and once by `load_with_provenance` for a three-layer stack.
10. Compile-time tests: `fn obj(_: &dyn LayerSource) {}` and
    `fn ss<T: Send + Sync>() {}` called with `Layers` and
    `Box<dyn LayerSource>` compile.

---

## REQ-CFG-0006: Built-in sources are configurable, not fixed

**Status:** Proposed  

### Requirement Statement

This item covers the sources `sc-config` ships. Each implements `LayerSource`
([REQ-CFG-0005](requirements.md)). Every setting a consumer might need to
change is a builder parameter whose default equals the behaviour of `develop`
at `9f61527` (the baseline before PR #153), not a constant. Builder methods
are infallible and take typed inputs; a value that can be invalid is checked
when the source loads and reported as a `ConfigError`, never a panic.

**`KeyPath` inputs.** Every path argument below is a `KeyPath`
([REQ-CFG-0007](requirements.md) item 2), so a path is validated once, at
`KeyPath` construction.

**`FileSource`**

1. `FileSource::new(path: impl Into<PathBuf>) -> FileSource` reads one file.
   `.optional()` makes an absent file return `Ok(None)`. By default an absent
   file is `ConfigError::MissingFile { path }` ([REQ-CFG-0009](requirements.md)).
   `Loader` maps its own `default.json` to `MissingDefault` instead
   ([REQ-CFG-0008](requirements.md) item 2).
2. "Absent" means `std::fs::symlink_metadata` reports `NotFound`. A symlink
   whose target is missing is not absent: it MUST be `ConfigError::Read`
   with the path, even when the source is optional.
3. Before opening, the source MUST check `std::fs::metadata(path)` and return
   `ConfigError::Read` with the path for anything that is not a regular file
   (directory, FIFO, socket, device). It MUST NOT open a non-regular file, so
   a FIFO never blocks the load.
4. `.max_bytes(n: u64)` sets a size cap. The default is no cap, as on
   `develop`; a consumer reading untrusted paths sets one. With a cap the
   source MUST read at most `n.saturating_add(1)` bytes from the opened
   handle and MUST return `ConfigError::TooLarge { path, limit: n }` when more
   than `n` bytes were read, so a file that grows after the metadata check is
   still refused. `n = u64::MAX` behaves as no cap and never overflows.
5. The default format is JSON (`serde_json::from_slice`). A JSON failure MUST
   be `ConfigError::Parse` with line and column, unchanged from
   [REQ-CFG-0004](requirements.md).
6. `.format(parser)` takes
   `impl Fn(&[u8]) -> Result<serde_json::Value, FormatError> + Send + Sync + 'static`,
   stored boxed, so a consumer can read TOML, YAML or anything else, with
   captured state, without `sc-config` depending on a parser
   ([NFR-CFG-0002](requirements.md) still binds). A parser failure MUST be
   `ConfigError::ParseFormat` carrying the path and the `FormatError`'s
   message, line and column.
7. `FormatError` is a public `#[non_exhaustive]` value returned by parsers,
   not an error enum: `FormatError::new(message: impl Into<String>)` and
   `.at(line: usize, column: usize)`. It implements `Display` and is
   converted into `ConfigError::ParseFormat` by the source.
8. `.accepts(paths: impl IntoIterator<Item = KeyPath>)` restricts the file to
   the given key paths. After parsing, every leaf path of the file value
   ([REQ-CFG-0007](requirements.md) item 3) that is not equal to or under an
   accepted path MUST be `ConfigError::UnknownKey { path, key }`, where
   `path` is the file and `key` is the first such leaf path in `KeyPath`
   order. Without `.accepts`, every key is accepted.
9. `source()` MUST return `Source::file(path)`.

**`EnvSource`**

10. `EnvSource::new(prefix: impl Into<String>, environment: Vec<(OsString, OsString)>) -> EnvSource`
    takes a literal prefix and an explicit environment. The prefix is
    matched literally and includes any trailing separator.
    `EnvSource::for_app(app: &AppName, environment) -> EnvSource` uses the
    prefix of [REQ-CFG-0002](requirements.md) item 1 (`AppName::env_prefix`).
    `EnvSource::from_process(prefix)` captures the process environment once
    through the crate's single `std::env::vars_os` call site. No other
    constructor reads the process environment.
11. Defaults reproduce [REQ-CFG-0002](requirements.md) exactly: segment
    separator `__`, segments lower-cased, values parsed as JSON else taken as
    a string, matching variables applied in ascending encoded-byte order of
    the name, a path through a non-object is `CFG.OVERRIDE`, an empty segment
    is `CFG.OVERRIDE`, an object value replaces whole.
12. The source applies its variables onto a working copy of `base` with the
    rules of [REQ-CFG-0002](requirements.md) items 11 to 13, and returns a
    `Layer` holding, for every path it set, the working copy's value at that
    path, marked `replace_at` that path and `attribute`d to the setting
    variable. The fold result therefore equals applying
    [REQ-CFG-0002](requirements.md) to `base` directly.
13. `.separator(s: impl Into<String>)` replaces `__`. An empty prefix or an
    empty separator MUST be `ConfigError::EnvSetting { setting, value }` at
    load.
14. `.key_case(KeyCase)` selects `KeyCase::Lower` (default) or
    `KeyCase::Preserve`, so mixed-case config keys are reachable from the
    environment. `KeyCase` is `#[non_exhaustive]`.
15. `.value_mode(ValueMode)` selects `ValueMode::Json` (default: JSON else
    string) or `ValueMode::String` (every value is a JSON string) for every
    prefixed variable. `ValueMode` is `#[non_exhaustive]`.
16. `.string_paths(paths: impl IntoIterator<Item = KeyPath>)`: a prefixed
    variable whose path is equal to or under one of `paths` MUST be taken as
    a JSON string whatever the value mode, so `DECIDR_LAYA__MODEL=1` sets the
    string `"1"`.
17. `.alias(variable: impl Into<String>, path: KeyPath)` maps one exact,
    case-sensitive variable name to one key path regardless of prefix, so a
    consumer can honour an established name such as `SC_AGENT_MODEL` →
    `model`. `.alias_string(variable, path)` does the same but always takes
    the value as a JSON string. An empty alias name MUST be
    `ConfigError::EnvSetting` at load.
18. A variable that is both prefixed and an alias MUST be applied only as
    the alias; [REQ-CFG-0002](requirements.md) items 2 and 3 apply to
    variables that are not aliases.
19. `.only(paths: impl IntoIterator<Item = KeyPath>)` restricts prefixed
    variables to the given paths: a prefixed variable whose path is equal to
    or under an entry applies; any other prefixed variable, including one
    whose path is an ancestor of an entry, MUST be ignored, not rejected. An
    empty segment is still `CFG.OVERRIDE` whether or not the path is in the
    set. Aliases are not filtered by `.only`.
20. Application order MUST be: prefixed variables in ascending encoded-byte
    order of the name, then aliases in ascending encoded-byte order of the
    name. The result never depends on the order of the input `Vec`. The
    rule "a longer name that extends a shorter one wins" holds where the
    shorter name is a byte-prefix of the longer one, which follows from that
    order.
21. A matching variable (prefixed or aliased) whose name or value is not
    Unicode MUST be `ConfigError::NonUnicode`. A non-matching variable MUST
    be ignored whatever its encoding.
22. `source()` MUST return `Source::env(prefix)`; each leaf it sets is
    attributed to `Source::env_var(name)` with the exact variable name.

**`ValueSource`**

23. `ValueSource::new(source: Source, value: serde_json::Value) -> ValueSource`
    contributes a caller-built value under the given `Source`. It is how a
    consumer feeds parsed command-line flags, compiled-in defaults or test
    fixtures into the stack. `sc-config` MUST NOT parse command lines.

### Rationale

The PR #153 loader fixed JSON, a 1 MiB cap, `__` and lower-case keys, and
three hard-coded loops for files, environment and flags. Each of those is a
choice one consumer made. As parameters with defaults equal to `develop`,
they cost current users nothing and leave every other consumer free to
choose. sc-agent named two needs PR #153 met that this item keeps: a per-file
key set that rejects unknown keys (`.accepts`, REQ-SAG-0019, ADR-SAG-0021) and
string-typed settings under a prefix (`.string_paths`, REQ-SAG-0020). A
caller-supplied parser keeps the two-dependency rule while removing the
JSON-only limit on consumers.

### Success Criteria

Tests use temporary directories from a temporary-directory API and in-memory
environments ([REQ-CFG-0003](requirements.md) items 10 and 11).

1. `FileSource::new(p).optional()` on an absent `p` returns `Ok(None)`;
   without `.optional()` it returns `MissingFile` carrying `p`.
2. On Unix, an optional `FileSource` on a symlink to a missing target returns
   `Read` carrying the path.
3. `FileSource` on a directory returns `Read` carrying the path. On Unix, a
   test creates a Unix socket with `std::os::unix::net::UnixListener::bind`
   (no extra crate) and asserts `Read` is returned without
   blocking, because the type check precedes `open`.
4. With `.max_bytes(4)`, a 5-byte file returns `TooLarge { limit: 4 }` and a
   4-byte file loads. A unit test inside `src/` drives the capped read
   through the source's internal `Read`-taking function with a reader that
   yields 5 bytes for a file whose metadata said 4, and asserts `TooLarge`.
   With `.max_bytes(u64::MAX)` a 5-byte file loads.
5. `.format(move |b| ...)` with a closure that captures a `String` and maps
   `b"k=v"` to `{"k":"v"}` loads `{"k":"v"}`; a parser returning
   `FormatError::new("bad").at(3, 1)` gives `ParseFormat` with that path,
   message, line 3 and column 1. Invalid JSON with the default format gives
   `Parse`.
6. `.accepts([model])` on `{"model":"x","log_root":"/l"}` returns
   `UnknownKey` carrying the file path and key `log_root`; on `{"model":"x"}`
   it loads.
7. Every success criterion of [REQ-CFG-0002](requirements.md) passes against
   `EnvSource::for_app(&my_app, env)` with default settings, including
   criterion 6 (`CFG.OVERRIDE` through `daemon.port`) and criterion 7
   (object value replaces), with the base supplied by a lower
   `ValueSource`.
8. `MY_APP__`, `MY_APP__A____B` and `MY_APP__A__` each return `Override`
   carrying the name. `MY_APP__DAEMON={"port":1,"host":"h"}` with
   `MY_APP__DAEMON__PORT=2` gives `{"daemon":{"port":2,"host":"h"}}`
   whatever the input order.
9. A prefixed variable with a non-Unicode value returns `NonUnicode` carrying
   the name; an unprefixed non-Unicode variable is ignored.
10. `.separator("_").key_case(KeyCase::Preserve)` on prefix `APP_` with
    `APP_Foo_Bar=1` sets `{"Foo":{"Bar":1}}`. `.separator("")` returns
    `EnvSetting`.
11. `.value_mode(ValueMode::String)` with `P__A=1` sets `{"a":"1"}`.
12. `.string_paths([model])` with `DECIDR_LAYA__MODEL=1` on prefix
    `DECIDR_LAYA__` sets `{"model":"1"}`, while `DECIDR_LAYA__N=1` sets
    `{"n":1}`.
13. `.alias("SC_AGENT_MODEL", model)` with `SC_AGENT_MODEL=x` sets
    `{"model":"x"}`; `.alias_string("V", v)` with `V=123` sets `{"v":"123"}`;
    with prefix `P__` and `.alias("P__X", y)`, `P__X=1` sets `{"y":1}` and no
    `x`.
14. `.only([a])` with `P__A=1,P__B=2` yields `{"a":1}` and no error;
    `.only([a.b])` with `P__A=1` yields `{}`.
15. Two loads with the same variables in opposite `Vec` order produce equal
    values.
16. An integration test reads `tests/fixtures/app-name-vectors.json` and
    asserts every accept vector constructs an `AppName` and every reject
    vector returns `InvalidName` with code `APP.INVALID_NAME`.

---

## REQ-CFG-0007: Per-leaf provenance with open source kinds

**Status:** Proposed  

### Requirement Statement

This item covers the provenance returned by `Layers::resolve` and
`Layers::load_with_provenance` ([REQ-CFG-0005](requirements.md)).

1. `Source` MUST be a public `#[non_exhaustive]` struct with `kind()` and
   `name()` accessors, `Display`, `Clone`, `Eq`, `Hash` and
   `serde::Serialize` (fields `kind` and `name`). `SourceKind` MUST be
   `#[non_exhaustive]` with `File`, `Env`, `Flag`, `Default` and
   `Custom(String)`. Constructors: `Source::file(path: &Path)`,
   `Source::env(prefix)`, `Source::env_var(name)`, `Source::flag(name)`,
   `Source::default_values(name)`, `Source::custom(kind, name)`. `Flag` and
   `Default` exist because sc-agent's provenance reports them; any other
   kind uses `Custom` with no change to `sc-config`. For a file,
   `Source::path() -> Option<&Path>` returns the exact path and `name()`
   is its lossless form when the path is Unicode and its
   `Path::display` form otherwise; equality and hashing use the exact path.
2. A key path MUST be a `KeyPath`: a public, owned, non-empty list of
   non-empty `String` segments.
   - `KeyPath::from_segments(impl IntoIterator<Item = impl Into<String>>) -> Result<KeyPath, ConfigError>`
     and `KeyPath::parse_dotted(&str) -> Result<KeyPath, ConfigError>` MUST
     return `ConfigError::InvalidKeyPath { input, reason }` for an empty path
     or an empty segment; they never panic. `TryFrom<&str>` is
     `parse_dotted`.
   - Grammar of the dotted form: segments separated by `.`; a segment is
     either a run of characters other than `.` and `"`, or a JSON string
     literal (which may contain `.`, `"` and escapes). `Display` writes a
     segment as a JSON string literal when it contains `.`, `"`, `\` or a
     control character, and bare otherwise, so `parse_dotted(&p.to_string())
     == Ok(p)` for every `KeyPath`.
   - `Serialize` writes a `KeyPath` as a JSON array of segment strings.
   - `Ord` is lexicographic over the segment vector, comparing segments by
     their UTF-8 bytes.
3. A leaf is a scalar, an array, `null` or an empty object in the resolved
   value. Arrays are leaves; their elements are not tracked separately,
   because arrays are replaced whole ([REQ-CFG-0001](requirements.md)).
4. `Provenance` MUST map every leaf path of the resolved value to the `Source`
   of the highest-precedence layer that set it: the `attribute`d source for
   that path when the layer gave one ([REQ-CFG-0005](requirements.md) item
   4), else the layer's `source()`. When an upper layer replaces an object
   with a non-object, or marks `replace_at`, the replaced leaves MUST
   disappear from the provenance. A resolved value that is itself a leaf
   (not an object) has no leaf paths and an empty provenance.
5. Provenance MUST be computed from the layer values during the fold. It MUST
   NOT deserialise any layer into `T`.
6. `Provenance::source_of(&self, path: &KeyPath) -> Option<&Source>` returns
   the source of that leaf, or `None` when the path is not a leaf.
   `Provenance::iter()` yields `(&KeyPath, &Source)` in `KeyPath` order.
7. `Provenance::rows(&self, value: &serde_json::Value) -> Vec<ResolvedRow>`
   (normally called with `Resolved::value()`) MUST return one row per leaf in
   `KeyPath` order, where `ResolvedRow` is `#[non_exhaustive]` and
   `Serialize`, with accessors `path()`, `source()` and `value()`; `value` is
   the typed JSON value, not a string. A path no layer set is absent; it is
   never reported as `Default`. `rows` MUST NOT fail and MUST NOT drop a
   leaf.

### Rationale

Provenance answers "where did this setting come from", which every consumer
with more than one layer eventually needs (`config show` in sc-agent, a
daemon's start-up log). A closed source enum and string-typed provenance
lock that answer to sc-agent's sources. Typed, serialisable rows let an agent
consume `config show --json` without reparsing. Runtime `KeyPath`s with a
defined grammar let keys come from data, including keys that contain `.`.

### Success Criteria

1. A test implements a source with `Source::custom("keychain", "login")` and
   asserts its leaves report that source.
2. Stack `[file {"a":{"b":1,"c":2}}, env P__A__B=3]`: `source_of(a.b)` is
   `env_var("P__A__B")`, `source_of(a.c)` is the file, and `rows` is
   `[a.b → P__A__B, 3; a.c → file, 2]`.
3. Stack `[{"a":{"b":1}}, {"a":5}]`: rows contain only `a`.
4. Table test of `parse_dotted` / `Display` round trips:
   `a.b` → `["a","b"]`; `"x.y"` → `["x.y"]`; `a."q\"d".c` → `["a","q\"d","c"]`;
   `"a\\b"` → `["a\\b"]`. `""`, `a..b`, `.a` and `a.` return `InvalidKeyPath`.
   `from_segments(Vec::<String>::new())` and `from_segments([""])` return
   `InvalidKeyPath`. No input panics, including every vector above passed to
   both constructors.
5. `{"a":[1,2]}` yields one row `a` with value `[1,2]`; `{"e":{}}` yields one
   row `e` with value `{}`.
6. `serde_json::to_value(rows)` succeeds; a row serialises with `path` as an
   array, and `source` with `kind` and `name` fields.
7. Rows for leaves `["b"]`, `["a","z"]`, `["a"," "]` come out in the order
   `["a"," "]`, `["a","z"]`, `["b"]`.

---

## REQ-CFG-0008: `Loader` is a preset stack on the layered engine

**Status:** Proposed  

### Requirement Statement

This item covers how `Loader` and the free function `load`
([REQ-CFG-0003](requirements.md)) relate to `Layers`
([REQ-CFG-0005](requirements.md)).

1. `Loader::load::<T>()` MUST be implemented as `self.layers().load::<T>()`.
2. `Loader::layers(&self) -> Layers` MUST be public and MUST return this
   stack, lowest first: `FileSource` of `<dir>/default.json` (required; its
   absence is reported as `ConfigError::MissingDefault`, code
   `CFG.MISSING_DEFAULT`, unchanged), `FileSource` of `<dir>/local.json`
   (optional), `EnvSource::for_app(&app, environment)` with default
   settings. The returned `Layers` owns a clone of the loader's environment.
3. A consumer MAY push further sources onto the returned `Layers`, for
   example a `ValueSource` of parsed flags.
4. The free function `load::<T>(app)` MUST be
   `Loader::new(app, "config".into(), process_environment()).load::<T>()`,
   where `process_environment()` is the crate's single `std::env::vars_os`
   call site, shared with `EnvSource::from_process`.
5. The crate MUST contain one file reader, one override applier and one
   `merge`. `Loader` MUST NOT keep a private pipeline.
6. Every behaviour required by [REQ-CFG-0001](requirements.md) through
   [REQ-CFG-0004](requirements.md) is unchanged, error variants and codes
   included.

### Rationale

Re-expressing `Loader` on the engine removes the PR #153 fork (two env
orders, two sets of file checks) and proves the engine can express the
crate's own conventions. Exposing `layers()` gives sc-agent and the
generated-project template one starting point to extend.

### Success Criteria

1. Every test in `crates/sc-config/tests` at `develop` `9f61527` passes with
   no edit to its file. New tests are added beside them; none replaces one.
2. A test compares `Loader::load::<Value>()` with
   `Loader::layers().load::<Value>()` over the REQ-CFG-0001 worked example
   and the REQ-CFG-0002 worked example and asserts equality.
3. A test pushes a `ValueSource` onto `Loader::layers()` and asserts it
   overrides an environment variable that sets the same key.
4. A test with a `default.json` larger than 1 MiB asserts `Loader` loads it
   (no cap by default).

---

## REQ-CFG-0009: One `ConfigError` covers every source, with codes

**Status:** Proposed  

### Requirement Statement

This item extends `ConfigError` ([REQ-CFG-0004](requirements.md)) to the
layered engine. The error inventory table of
[ADR-CFG-0001](architecture.md) is the only list of variants, codes and
recovery steps; this item states the obligations and does not repeat the
table.

1. Every fallible public function of the engine, of `KeyPath` and of every
   built-in source MUST return `ConfigError`. `FormatError`
   ([REQ-CFG-0006](requirements.md) item 7) is a parser return value, not an
   error enum, and is always converted to `ConfigError::ParseFormat`. The
   crate MUST export no other error type.
2. `ConfigError` MUST be `#[non_exhaustive]` at the enum level. Its variants
   are not individually `#[non_exhaustive]`, so callers can still construct
   them.
3. `ConfigError::code(&self) -> &'static str` and
   `ConfigError::recovery(&self) -> &'static str` are public and cover every
   variant, new ones included.
4. New variants, with the codes and recovery steps of the ADR-CFG-0001
   inventory: `MissingFile { path }` (`CFG.MISSING_FILE`),
   `TooLarge { path, limit }` (`CFG.TOO_LARGE`),
   `ParseFormat { path, message, line: Option<usize>, column: Option<usize> }`
   (`CFG.PARSE_FORMAT`), `UnknownKey { path, key: KeyPath }`
   (`CFG.UNKNOWN_KEY`), `InvalidKeyPath { input, reason }` (`CFG.KEY_PATH`),
   `EnvSetting { setting, value }` (`CFG.ENV_SETTING`) and
   `SourceFailed { origin: Source, message, cause }` (`CFG.SOURCE`).
   `MissingDefault` and `CFG.MISSING_DEFAULT` are unchanged and remain
   `Loader`'s report for an absent `default.json`.
5. `SourceFailed` is how an external `LayerSource` reports failure:
   `ConfigError::source_failed(origin: Source, message: impl Into<String>)`
   and `ConfigError::source_failed_with(origin, message, cause: Box<dyn std::error::Error + Send + Sync>)`.
   `Error::source()` returns the cause when one was given. The field is
   named `origin` so it does not shadow `Error::source`.
6. `Deserialize` MUST additionally carry `sources: Vec<Source>`: every source
   whose `load` returned `Some`, in precedence order. Its `files` field MUST
   be the paths of the `File`-kind sources among them, and its `variables`
   field the names of the environment variables that were applied (the
   `attribute`d `env_var` sources), so the existing fields are derived from
   `sources` and keep [REQ-CFG-0004](requirements.md) criterion 7.
7. `ConfigError::report(&self) -> serde_json::Value` MUST return an object
   with `code`, `message` (the `Display` text), `recovery` and `fields` (the
   variant's typed fields as JSON; paths as strings via `Path::display`), so
   an agent can consume an error without parsing text.
8. Every variant's `Display` MUST name the path, variable, key path or source
   it carries. `ConfigError` stays `Send + Sync`.

### Rationale

PR #153's `LayersError` had no codes, no recovery text and free-form
messages, breaking the actionable-error contract of REQ-CFG-0004 for the new
path. One error type keeps one contract. Enum-level `#[non_exhaustive]` is
the only way to add the next failure mode without breaking every caller,
which ADR-CFG-0001 already lists as a cost. Keeping `MissingDefault` keeps
the stable code that scripts and the template daemon already use.

### Success Criteria

1. An integration test outside the crate matches on `ConfigError` with a
   wildcard arm, and constructs each new variant by literal; both compile.
2. For each new variant a test triggers it through the public API and
   asserts `code()`, a non-empty `recovery()`, that `to_string()` contains
   the carried path, variable, key path or source name, and that
   `report()["code"]` equals `code()`.
3. A custom source returning
   `ConfigError::source_failed_with(Source::custom("k","n"), "denied", Box::new(io_err))`
   surfaces as `SourceFailed` with that origin and message, and
   `std::error::Error::source` returns the I/O error.
4. A deserialise failure over the stack `[file, env var applied, value]`
   carries all three in `sources` in order, the file in `files` and the
   variable in `variables`.
5. `fn ss<T: Send + Sync>() {}` called with `ConfigError` compiles.

---

## REQ-CFG-0010: `StandardLayers` fixes the recommended layer order

**Status:** Proposed  

### Requirement Statement

This item covers the standard layer order of the `sc-config` crate
(`crates/sc-config`) and the builder that enforces it. The decision is
[ADR-CFG-0005](architecture.md). `StandardLayers` builds a `Layers` stack
([REQ-CFG-0005](requirements.md)) from the built-in sources
([REQ-CFG-0006](requirements.md)); it adds no second pipeline.

**Slots.** `Slot` MUST be a public `#[non_exhaustive]` enum with exactly
these seven variants. Its rank is fixed, lowest precedence first:

| Rank | `Slot` | Content | Absent folder or file |
|---|---|---|---|
| 1 | `Defaults` | one `ValueSource` built by the consumer | - |
| 2 | `System` | files in the system folder | skipped |
| 3 | `User` | files in the user folders | skipped |
| 4 | `Project` | files in the project config folder | required unless `.optional()` |
| 5 | `Local` | `local.json` in the project config folder, then files in the work-tree folder | skipped |
| 6 | `Env` | one `EnvSource` | - |
| 7 | `Flags` | one `ValueSource` of flags parsed by the consumer | - |

1. `StandardLayers::new(app: AppName, context: StandardContext) -> StandardLayers`
   creates a builder with every slot empty.
2. `StandardContext` MUST be a public `#[non_exhaustive]` struct built by
   `StandardContext::new(environment: Vec<(OsString, OsString)>, project_root: PathBuf, platform: Platform)`.
   `StandardContext::from_process(project_root)` is the convenience that
   captures `std::env::vars_os()` through the crate's single call site
   ([REQ-CFG-0008](requirements.md) item 4) and uses `Platform::current()`.
   `Platform` MUST be a public `#[non_exhaustive]` enum with `Unix` and
   `Windows`; `Platform::current()` returns `Windows` under
   `cfg(windows)` and `Unix` otherwise. No locator reads the process
   environment, the current directory or `cfg` itself: every path below is
   computed from the `StandardContext` alone.
3. One method per slot, each returning `Result<StandardLayers, ConfigError>`:
   `.defaults(value: serde_json::Value)`, `.system(files: FileNames)`,
   `.user(user: UserFolders, files: FileNames)`,
   `.project(files: FileNames)`, `.local(local: LocalFolders, files: FileNames)`,
   `.env(source: EnvSource)`, `.flags(value: serde_json::Value)`.
4. Calling a slot method on a slot that is already filled MUST return
   `ConfigError::DuplicateSlot { slot }` (code `CFG.DUPLICATE_SLOT`) and
   leave nothing changed.
5. The order in which slot methods are called MUST NOT affect the result:
   the stack is always assembled in slot rank. Every slot is optional to
   fill; an unfilled slot contributes nothing.
6. `.insert(after: After, source: Box<dyn LayerSource>) -> StandardLayers`
   is the only way to add a consumer-specific source. `After` MUST be a
   public `#[non_exhaustive]` enum with `Defaults`, `System`, `User`,
   `Project`, `Local` and `Env`; there is no `After::Flags`, so flags are
   always the highest layer. Inserted sources sit directly above their
   named slot, in insertion order, whether or not that slot is filled.
7. `StandardLayers` MUST NOT offer any method that reorders, swaps or
   re-ranks slots.
8. `.build() -> Layers` returns the assembled stack;
   `.load::<T>()`, `.resolve()` and `.load_with_provenance::<T>()` are
   `self.build()` followed by the `Layers` method of the same name.

**File names within a slot.**

9. `FileNames` MUST be a public type built by
   `FileNames::new(names: impl IntoIterator<Item = impl Into<String>>) -> Result<FileNames, ConfigError>`:
   an ordered, non-empty list of file names, lowest precedence first (for
   example `["sc-agent.json", "decidr.json"]`: `decidr.json` wins). A name
   that is empty, is `.` or `..`, or contains `/` or `\` MUST return
   `ConfigError::InvalidFileName { name }` (code `CFG.FILE_NAME`). A
   duplicate name MUST return the same error.
10. Within one slot the order MUST be folder-major: every listed file of the
    lower folder, in list order, then every listed file of the higher
    folder, in list order. Each file is one `FileSource`, optional except in
    `Project`.

**Locators.** Each folder below is computed from `StandardContext`. An
environment value is used only when it is present, Unicode, non-empty and
an absolute path; otherwise it counts as unset. `<app>` is
`AppName::as_str`.

11. `System`: `Unix` → `/etc/<app>/`. `Windows` → `%ProgramData%\<app>\`;
    when `ProgramData` is unset the slot reads nothing. `XDG_CONFIG_DIRS`
    is not read.
12. `User` folders, lower first, according to `UserFolders`, a public
    `#[non_exhaustive]` enum:
    - `UserFolders::Config` (the default the preset uses): the config folder
      only. `Unix` → `$XDG_CONFIG_HOME/<app>/`, else `$HOME/.config/<app>/`.
      `Windows` → `%APPDATA%\<app>\`.
    - `UserFolders::Dotfolder(name: String)`: the home dotfolder only,
      `<home>/.<name>/`, where `<home>` is `$HOME`, else on `Windows`
      `%USERPROFILE%`. `name` follows the `FileNames` name rule.
    - `UserFolders::ConfigThenDotfolder(name)`: both, config folder lower,
      dotfolder higher.
    When the needed variable is unset, that folder is skipped.
13. `Project`: `<project_root>/config/`.
14. `Local` folders, lower first, according to `LocalFolders`, a public
    `#[non_exhaustive]` enum:
    - `LocalFolders::ConfigLocal`: `<project_root>/config/local.json` only;
      the `files` argument is ignored for this form and MAY be
      `FileNames::new(["local.json"])`.
    - `LocalFolders::WorkTree`: `<worktree>/.sc/<app>/` only.
    - `LocalFolders::ConfigLocalThenWorkTree` (the default the preset uses):
      `config/local.json` first, then the work-tree folder.
15. Work-tree detection MUST use `std::fs` only and run no process: starting
    at `project_root` and moving to each parent in turn, the first directory
    that contains an entry named `.git` (a directory, or a file as in a
    linked worktree or submodule) is `<worktree>`. The walk stops after the
    filesystem root, or after the directory set by
    `StandardContext::with_worktree_ceiling(dir)` when one is set (the
    ceiling itself is checked). With no match there is no work tree and the
    work-tree folder is skipped. An I/O error other than not-found while checking an entry
    MUST return `ConfigError::Read` with that path.

**Provenance and preset.**

16. Every `Source` produced by `StandardLayers` MUST carry its slot:
    `Source::slot() -> Option<Slot>` returns it, and `Serialize` adds a
    `slot` field when present. Inserted sources report `None` unless they set
    one.
17. `StandardLayers::recommended(app, context)` MUST be the documented
    preset: it fills `System` and `Project` with `FileNames` `["config.json"]`
    and `["default.json"]`, `User` with `UserFolders::Config` and
    `["config.json"]`, `Local` with `LocalFolders::ConfigLocalThenWorkTree`
    and `["config.json"]`, and `Env` with `EnvSource::for_app(&app,
    environment)`. `Project` is optional in the preset. `Defaults` and
    `Flags` are left for the consumer, who alone holds those values; the
    rustdoc example fills them.
18. `Loader::standard(&self) -> StandardLayers` MUST return the stack of
    `Loader::layers()` ([REQ-CFG-0008](requirements.md)) expressed as slots:
    `Project` (required, `["default.json"]`, folder = the loader's config
    directory, absence reported as `MissingDefault`), `Local` with
    `LocalFolders::ConfigLocal`, and `Env`. It MUST NOT include `User`,
    `System` or the work-tree folder; `Loader`'s behaviour is unchanged.

### Rationale

Without one order, every product that builds its own stack decides
precedence and paths again, and users of two products see two answers to
"which file wins". Slots fix the order and the locators fix the paths, so
consumers choose which layers they use and which file names live in them,
but not the order. Folder-major order within a slot keeps a folder's files
together, and the more specific folder (dotfolder over config folder,
work tree over project) wins, matching the slot order's
general-to-specific rule. Paths come from an explicit environment so tests
are deterministic and nothing reads process state behind the caller.

### Success Criteria

All tests live in `crates/sc-config`, build a `StandardContext` with an
in-memory environment, a temporary `project_root` and an explicit
`Platform`, and run under `cargo test`. No test depends on the host's
`HOME` or platform.

1. Filling all seven slots in rank order and in reverse rank order, with
   each slot setting `{"k": <rank>}`, both load `{"k": 7}`; resolving each
   reports `k` from `Slot::Flags`.
2. A second `.env(...)` call returns `DuplicateSlot { slot: Slot::Env }`
   with code `CFG.DUPLICATE_SLOT`.
3. `.insert(After::User, s)` with `s` setting `{"k":"ins"}`, `User` setting
   `{"k":"user"}` and `Project` setting `{"k":"proj"}` loads `"proj"`;
   without `Project` it loads `"ins"`. Two inserts at `After::User` apply in
   insertion order.
4. `FileNames::new(["a.json","b.json"])` in one folder, both setting `k`,
   loads `b.json`'s value. `FileNames::new([""])`, `["../x"]`, `["a/b"]`
   and `["a","a"]` each return `InvalidFileName`.
5. `UserFolders::ConfigThenDotfolder("sc-agent")` with `XDG_CONFIG_HOME`
   and `HOME` set to temporary folders, files `["sc-agent.json",
   "decidr.json"]` present in both: the dotfolder's `decidr.json` wins, and
   resolving reports each key's exact path.
6. `Unix`: with `XDG_CONFIG_HOME` unset the user folder is
   `$HOME/.config/<app>/`; with `XDG_CONFIG_HOME=relative/path` it is also
   `$HOME/.config/<app>/`. `Windows`: the user folder is
   `%APPDATA%\<app>\`, and with `HOME` unset the dotfolder uses
   `%USERPROFILE%`. The locator's computed paths are asserted directly.
7. `System` on `Unix` computes `/etc/<app>/` and on `Windows` with
   `ProgramData` unset reads nothing. A test asserts the computed path
   without reading `/etc`.
8. Work tree: a temporary tree `root/.git/` (directory) with
   `project_root = root/a/b` finds `root`; with `.git` as a file it finds
   `root`; with no `.git` and `with_worktree_ceiling(temp_root)`, no work
   tree is found and the folder is skipped, whatever the host's own
   directories contain.
9. `LocalFolders::ConfigLocalThenWorkTree` with `config/local.json` and
   `<worktree>/.sc/<app>/config.json` both setting `k` loads the work-tree
   value.
10. `Loader::standard()` and `Loader::layers()` resolve equal values and
    equal provenance over the REQ-CFG-0001 worked example, and an absent
    `default.json` returns `MissingDefault` from both.
11. `serde_json::to_value` of a row from `StandardLayers` has
    `source.slot`; a row from an inserted source has none.
12. A `compile_fail` doctest using `After::Flags` fails to compile, and a
    doctest inserting at each of the six `After` variants compiles.

---

## NFR-CFG-0001: Public API returns `Result<T, ConfigError>`; no panics

**Status:** Active  

### Requirement Statement

This item binds all code under `crates/sc-config/src`.

The panic rule for the four library crates of this repository is owned by
[NFR-RUN-0009](../requirements.md). That rule is: code reachable from a public
function MUST NOT use `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`,
`unimplemented!`, or panicking `[]` indexing. There is no allowance for a
commented exception. Test code is exempt. This item applies that rule to
`sc-config` and adds the crate-specific entries 5 and 6.

1. Every public function and public method of `sc-config` that can fail MUST
   return `Result<T, ConfigError>`, where `ConfigError` is the crate's public
   typed error enum ([REQ-CFG-0004](requirements.md)). In Rust this is the
   discriminated union the design calls for: the caller receives either the
   value or a typed error and must handle both.
2. No public function or method may return `Option` to signal failure, return
   a boxed or opaque error (`Box<dyn Error>`, `anyhow::Error`), or report
   failure by printing or logging.
3. No code path reachable from a public function or method may panic, for any
   input: any app name, any directory path, any file content, any environment
   content including variables that are not valid Unicode.
4. Code reachable from a public function MUST NOT use `unwrap`, `expect`,
   `panic!`, `unreachable!`, `todo!`, `unimplemented!`, or panicking `[]`
   indexing ([NFR-RUN-0009](../requirements.md)). Where a slice, string, `Vec`
   or `serde_json::Value` is accessed, the code MUST use `get`, pattern
   matching or iterators.
5. Stricter than the repository rule, decided in this document: code reachable
   from a public function MUST NOT use `assert!`, `assert_eq!` or
   `assert_ne!`.
6. Stricter than the repository rule, decided in this document: non-test code
   MUST NOT call `std::env::vars()`, which panics on a variable that is not
   valid Unicode. It MUST use `std::env::vars_os()`.
7. Implementations of standard traits on public types (`Display`, `Debug`,
   `std::error::Error`, `From`) are not "public methods" for entry 1, but
   entries 3, 4 and 5 apply to them.
8. Code inside `#[cfg(test)]` modules and under `crates/sc-config/tests/` is
   exempt from entries 4 and 5.

**Decided:** Only an operation that can fail returns `Result<T, ConfigError>`.
`Loader::new` stores a validated `AppName` plus the caller's directory and
environment, so it returns `Loader`. `AppName::new` can fail and returns
`Result<AppName, ConfigError>`. [NFR-RUN-0009](../requirements.md) requires
`Result` of operations that can fail.

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

## NFR-CFG-0002: `sc-config` depends only on `serde` and `serde_json`

**Status:** Active  

### Requirement Statement

This item binds `crates/sc-config/Cargo.toml`. It is the owner of the list of
third-party crates `sc-config` may depend on. The edges between the crates of
this workspace, the forbidden edges, and the boundary manifest under
`boundaries/sc-config/` that encodes them are owned by
[REQ-RUN-0005](../requirements.md), which refers to this item for the
third-party list.

1. The `[dependencies]` table of `crates/sc-config/Cargo.toml` MUST contain
   exactly two entries: `serde` and `serde_json`.
2. `[dependencies]` MUST NOT contain optional dependencies or
   target-specific dependency tables that add a third crate.
3. The `ConfigError` type MUST be written by hand. `thiserror` and `anyhow`
   MUST NOT be added.
4. The forbidden edges of `sc-config` are `sc-observability`,
   `sc-observability-otlp` (the OTel export crate of `sc-observability`),
   `tokio`, `sc-transport`, `sc-command` and `sc-runtime`. The obligation,
   and the `forbidden_edges` entries of the boundary manifest that
   `sc-lint-boundary` checks, are stated in
   [REQ-RUN-0005](../requirements.md). This item adds only the `cargo tree`
   check in criterion 3.

**Decided:** `[dev-dependencies]` is empty. Tests create a private directory
under `std::env::temp_dir` and remove it when the test ends. No third test
crate is added.

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

1. Inspection of `crates/sc-config/Cargo.toml`: the `[dependencies]` table has
   the keys `serde` and `serde_json` and no others, and the file has no
   `[target.*.dependencies]` table.
2. `cargo tree -p sc-config -e normal --depth 1` lists `serde` and
   `serde_json` as the only direct dependencies.
3. `cargo tree -p sc-config -e normal --prefix none` prints no line beginning
   with `<name> ` (the crate name followed by a space) for each of these
   names: `sc-observability`, `sc-observability-otlp`, `tokio`,
   `sc-transport`, `sc-command`, `sc-runtime`, `thiserror`, `anyhow`. The
   match is on the start of the line because the first line of the output
   holds the checkout path, which may itself contain `sc-runtime`.
4. The boundary manifest under `boundaries/sc-config/` and the
   `sc-lint-boundary` run are checked by the criteria of
   [REQ-RUN-0005](../requirements.md).

---

## NFR-CFG-0003: `sc-config` API is synchronous in v0.1

**Status:** Active  

### Requirement Statement

This item binds all code under `crates/sc-config/src` for the first release
(v0.1).

1. Every public function and method of `sc-config` MUST be synchronous. A call
   to `load` or `Loader::load` reads the files, applies the overrides,
   deserialises, and returns on the calling thread.
2. The crate MUST NOT contain an `async fn`, an `.await`, or a public function
   that returns a `Future`.
3. The crate MUST NOT watch files for changes.
4. The crate MUST NOT reload configuration after a load has returned.
5. The crate MUST NOT offer change notification: no callbacks, channels or
   subscriptions.
6. The crate MUST NOT spawn threads or tasks.
7. Automatic reloading, change notification and async interop are out of
   scope for v0.1. If they are built later they SHOULD go into a separate
   crate, possibly named `sc-config-tokio`, that depends on `sc-config`.

### Rationale

The design records automatic reloading, change notification and async interop
as a note for later, possibly as a separate `sc-config-tokio` crate, and
states they are not in the initial plan. Building them now would bring an
async runtime and a file watcher into every program that reads its
configuration once at start-up, which is every current user. A synchronous
function can be called from `main` before a runtime exists and from inside an
async `main` alike.

### Success Criteria

1. `grep -rnE "async fn|\.await|impl Future|dyn Future" crates/sc-config/src`
   returns no match.
2. `grep -rn "thread::spawn" crates/sc-config/src` returns no match, and
   `grep -nE "tokio|async-std|futures|notify" crates/sc-config/Cargo.toml`
   returns no match.
3. A test calls `Loader::load` from a plain `#[test]` function with no async
   runtime and asserts it returns `Ok`.
4. A test loads, then rewrites `default.json` with a different value, and
   asserts the value returned by the first load is unchanged and that a second
   explicit load returns the new value.

---

## NFR-CFG-0004: Public types can grow without breaking callers

**Status:** Proposed  

### Requirement Statement

This item binds every public type of `crates/sc-config/src` added or changed
by [REQ-CFG-0005](requirements.md) through [REQ-CFG-0009](requirements.md).

1. Every public enum MUST be `#[non_exhaustive]` at the enum level.
2. Every public struct MUST be `#[non_exhaustive]` or have only private
   fields, and MUST be constructible from outside the crate through a
   constructor or builder.
3. Configuration of a source or stack MUST be by builder methods with
   defaults, not by a struct literal the caller fills in.
4. No public signature may take `&'static str` where a runtime value is a
   legitimate input (key paths, variable names, prefixes, separators).
5. Every item exported from `crates/sc-config/src/lib.rs` MUST be named in
   the `[public] facade` of `boundaries/sc-config/sc-config.toml`; the
   existing test `facade_lists_every_export`
   (`crates/sc-config/tests/facade.rs`) is the check.
6. A method added to a public unsealed trait (`LayerSource`) MUST have a
   default body.

### Rationale

PR #153's `Inputs`, `Key`, `Layered` and `Resolved` were exhaustive structs
with public fields, so the next field any consumer needs breaks every other
consumer. `Key.path: &'static str` stopped keys being built from data. An
unsealed trait without a default-body rule breaks every implementer on its
next method.

### Success Criteria

All checks run under `cargo test -p sc-config`; no separate script or tool.

1. An integration test under `crates/sc-config/tests/` constructs every
   public struct through its constructor or builder (no struct literal) and
   matches every public enum with a wildcard arm. A struct missing
   `#[non_exhaustive]` with public fields is caught in review against item
   2; a struct or enum that cannot be built or matched this way fails to
   compile.
2. The same test builds every key path, prefix, separator and alias from a
   `String` read at runtime, which compiles only if no signature requires
   `&'static str`.
3. `facade_lists_every_export` passes with the facade naming every new
   export.
4. A test implements `LayerSource` with only `source` and `load`, so a
   method added without a default body fails to compile.
