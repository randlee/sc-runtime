# `sc-config` Requirements

**ID Range:** REQ-CFG-0001 through REQ-CFG-0009; NFR-CFG-0001 through NFR-CFG-0004  
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
| REQ-CFG-0001 through REQ-CFG-0009 | Requirements | REQ-CFG-0005..0009 Proposed (layered engine, ADR-CFG-0004) |
| NFR-CFG-0001 through NFR-CFG-0004 | Non-functional requirements | NFR-CFG-0004 Proposed |

---

## REQ-CFG-0001: `load` merges `default.json` and `local.json`

**Status:** Active  

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
[ADR-CFG-0004](architecture.md).

1. The crate MUST export a public trait `LayerSource` with these methods:

   ```rust
   pub trait LayerSource {
       /// Provenance of every value this source contributes.
       fn source(&self) -> Source;
       /// `Ok(None)` means "absent, contributes nothing" (for example an
       /// optional file that does not exist). It is not an error.
       fn load(&self) -> Result<Option<serde_json::Value>, ConfigError>;
   }
   ```

2. The trait MUST be object safe and MUST NOT be sealed, so a crate outside
   `sc-config` can implement it for its own source (a keychain, a remote
   store, a CLI profile).
3. The crate MUST export a builder type `Layers` that holds an ordered list of
   `Box<dyn LayerSource>`. `Layers::new()` creates an empty stack and
   `Layers::push(source)` appends a source as the new highest-precedence
   layer. Precedence is exactly the push order: a later source overrides an
   earlier one.
4. `sc-config` MUST NOT impose, reorder or insert layers. Whatever order the
   caller pushes is the order applied. No source kind (file, environment,
   flag, default) has a built-in rank.
5. `Layers::load::<T>()` for `T: serde::de::DeserializeOwned` MUST:
   1. call `load` on every source once, lowest precedence first;
   2. skip a source that returns `Ok(None)`;
   3. stop and return the first `Err`, unchanged;
   4. fold the returned values with `merge(base, overlay)` exactly as defined
      in [REQ-CFG-0001](requirements.md), starting from an empty JSON object;
   5. deserialise the final value into `T` exactly once with
      `serde_json::from_value::<T>` ([ADR-CFG-0002](architecture.md) items 4
      and 5 still bind: no layer is deserialised into `T` on its own).
6. `Layers::load_with_provenance::<T>()` MUST do the same and also return the
   per-leaf provenance of [REQ-CFG-0007](requirements.md), as
   `Layered<T> { value: T, provenance: Provenance }`.
7. An empty stack, or a stack whose sources all return `Ok(None)`, MUST
   deserialise `{}` into `T`. It is not an error.
8. The engine MUST NOT read the process environment, the current directory or
   any file itself. Only sources do I/O.

### Rationale

PR #153 added a second loader, beside `Loader`, with its own file checks, its
own environment ordering and a fixed flags > environment > files precedence.
Two pipelines drift, and a fixed precedence serves one consumer. A single
engine over caller-ordered sources lets each consumer state its own order
(sc-agent's flags over environment over files, a daemon's files over
environment, a test's in-memory values) without `sc-config` knowing any of
them. Keeping the fold on `serde_json::Value` with the existing `merge`
preserves ADR-CFG-0002.

### Success Criteria

Tests live in `crates/sc-config`. Sources in tests are in-memory
`ValueSource`s ([REQ-CFG-0006](requirements.md)) unless stated.

1. Stack `[{"a":1,"b":1}, {"a":2}]` loads into `Value` as `{"a":2,"b":1}`;
   the same two sources pushed in reverse order load as `{"a":1,"b":1}`.
2. A test defines a struct outside the crate's modules that implements
   `LayerSource` (returns `{"x":"custom"}`), pushes it, and asserts the load
   yields `{"x":"custom"}`. The test compiles only because the trait is
   public and unsealed.
3. A source returning `Ok(None)` between two value sources leaves the result
   equal to the stack without it.
4. A source returning `Err(e)` makes `load` return that same error variant;
   a counting source pushed after it asserts its `load` was never called.
5. An empty stack loads `#[derive(Deserialize, Default)] #[serde(default)]
   struct C { a: u8 }` as `C { a: 0 }`.
6. A test with a `T` whose `Deserialize` counts its invocations asserts it is
   called once for a three-layer stack.
7. `fn assert_obj_safe(_: &dyn LayerSource) {}` compiles.

---

## REQ-CFG-0006: Built-in sources are configurable, not fixed

**Status:** Proposed  

### Requirement Statement

This item covers the sources `sc-config` ships. Each implements `LayerSource`
([REQ-CFG-0005](requirements.md)). Every setting a consumer might need to
change is a builder parameter with a default, not a constant.

**`FileSource`**

1. `FileSource::new(path)` reads one file. `.optional()` makes an absent file
   return `Ok(None)`; by default an absent file is `ConfigError::MissingFile`
   ([REQ-CFG-0009](requirements.md)).
2. The file MUST be a regular file after following symlinks. Anything else
   (directory, device, FIFO) MUST be `ConfigError::Read` with the path.
3. `.max_bytes(n)` sets a size cap. The default is no cap. With a cap, the
   source MUST read at most `n + 1` bytes from the opened handle and MUST
   return `ConfigError::TooLarge { path, limit }` when more than `n` bytes
   exist, so a file that grows after a metadata check is still refused.
4. The default format is JSON (`serde_json::from_slice`).
   `.format(parser)` replaces it with a caller-supplied
   `fn(&[u8]) -> Result<serde_json::Value, FormatError>`, so a consumer can
   read TOML, YAML or anything else without `sc-config` depending on a
   parser ([NFR-CFG-0002](requirements.md) still binds).
   `FormatError { message, line: Option<usize>, column: Option<usize> }` is
   public and constructible by callers.
5. `source()` MUST return `Source::file(path)`.

**`EnvSource`**

6. `EnvSource::new(prefix, environment)` takes a prefix string and an explicit
   `Vec<(OsString, OsString)>`. It MUST NOT read the process environment;
   `EnvSource::from_process(prefix)` is the convenience that captures
   `std::env::vars_os()` once.
7. Defaults reproduce [REQ-CFG-0002](requirements.md) exactly: segment
   separator `__`, each segment lower-cased, value parsed as JSON else taken
   as a string, a longer name that extends a shorter one wins.
8. `.separator(s)` replaces `__`. `.key_case(KeyCase)` selects
   `Lower` (default), `Preserve` or `Upper` so upper-case or mixed-case config
   keys are reachable from the environment. `KeyCase` is `#[non_exhaustive]`.
9. `.alias(variable, path)` maps one exact variable name to one key path
   regardless of prefix, so a consumer can honour an established name such as
   `SC_AGENT_MODEL` → `model`. `.alias_string(variable, path)` does the same
   but always takes the value as a string, never parsing JSON.
10. `.only(paths)` restricts the source to the given key paths: a prefixed
    variable whose path is not in the set (or under one of its entries) MUST
    be ignored, not rejected. Without `.only`, every prefixed variable
    applies.
11. Matching variables MUST be applied in ascending encoded-byte order of the
    variable name, with aliases applied after prefixed variables, so the
    result never depends on the order of the input `Vec`.
12. A matching variable (prefixed or aliased) whose name or value is not
    Unicode MUST be `ConfigError::NonUnicode`. A non-matching variable MUST
    be ignored whatever its encoding.
13. `source()` MUST return `Source::env(prefix)`; each leaf's provenance names
    the exact variable that set it ([REQ-CFG-0007](requirements.md)).

**`ValueSource`**

14. `ValueSource::new(source, value)` contributes a caller-built
    `serde_json::Value` under the given `Source`. It is how a consumer feeds
    parsed command-line flags, compiled-in defaults or test fixtures into the
    stack. `sc-config` MUST NOT parse command lines itself.

### Rationale

The PR #153 loader fixed JSON, a 1 MiB cap, `__` and lower-case keys, and
three hard-coded loops for files, environment and flags. Each of those is a
choice one consumer made. As parameters with defaults equal to today's
behaviour, they cost nothing for current users and leave every other consumer
free to choose. A caller-supplied parser keeps the two-dependency rule while
removing the JSON-only limit on consumers.

### Success Criteria

1. `FileSource::new(p).optional()` on an absent `p` returns `Ok(None)`;
   without `.optional()` it returns `MissingFile` carrying `p`.
2. `FileSource` on a directory returns `Read` carrying the path.
3. With `.max_bytes(4)`, a 5-byte file returns `TooLarge { limit: 4 }`; a
   4-byte file loads. A test using a `Read` wrapper that yields more bytes
   than `metadata().len()` reported asserts `TooLarge` (no reliance on a
   race).
4. `.format(|b| ...)` with a parser that maps `b"k=v"` to `{"k":"v"}` loads
   `{"k":"v"}`; a parser error returns `ParseFormat` carrying the path and
   the parser's message.
5. Every success criterion of [REQ-CFG-0002](requirements.md) passes against
   `EnvSource::new("MY_APP__", env)` with default settings.
6. `.separator("_").key_case(KeyCase::Preserve)` with `APP_Foo_Bar=1` sets
   `{"Foo":{"Bar":1}}`.
7. `.alias("SC_AGENT_MODEL", "model")` with `SC_AGENT_MODEL=x` sets
   `{"model":"x"}`; `.alias_string("V", "v")` with `V=123` sets `{"v":"123"}`.
8. `.only(["a"])` with `P__A=1,P__B=2` yields `{"a":1}` and no error.
9. Two calls with the same variables in opposite `Vec` order produce equal
   values for `P__A={"b":1}` and `P__A__B=2`.
10. A prefixed variable with a non-Unicode value returns `NonUnicode`; an
    unprefixed non-Unicode variable is ignored.
11. `grep -rn "std::env::var" crates/sc-config/src` matches only
    `from_process` and the free `load`.

---

## REQ-CFG-0007: Per-leaf provenance with open source kinds and redaction

**Status:** Proposed  

### Requirement Statement

This item covers the provenance returned by
`Layers::load_with_provenance` ([REQ-CFG-0005](requirements.md)).

1. `Source` MUST be a public `#[non_exhaustive]` struct with a kind and a
   name, `Display`, `Clone`, `Eq`, `Hash` and `serde::Serialize`.
   `SourceKind` MUST be `#[non_exhaustive]` with at least `File`, `Env`,
   `Flag`, `Default` and `Custom(String)`. Constructors: `Source::file(path)`,
   `Source::env(name)`, `Source::flag(name)`, `Source::default_values(name)`,
   `Source::custom(kind, name)`. A consumer's own source kind is expressible
   with `Custom` without a change to `sc-config`.
2. A key path MUST be a `KeyPath`: an owned list of segments, built at
   runtime (`KeyPath::from_segments`, `KeyPath::parse_dotted`). A segment may
   contain `.`; `Display` writes segments joined by `.` with any segment
   containing `.` or `"` written as a JSON string literal. `parse_dotted`
   accepts that same form, so `Display` and `parse_dotted` round-trip.
3. A leaf is a scalar, an array, `null` or an empty object in the merged
   value. Arrays are leaves; their elements are not tracked separately.
4. `Provenance` MUST map every leaf path of the merged value to the `Source`
   of the highest-precedence layer that set it. For an environment variable
   or alias the recorded `Source` MUST carry the exact variable name. When an
   upper layer replaces an object with a non-object, the object's former
   leaves MUST disappear from the provenance.
5. Provenance MUST be computed from the layer values during the fold. It MUST
   NOT deserialise any layer into `T`.
6. `Provenance::rows()` MUST return `Vec<ResolvedRow>` sorted by path, where
   `ResolvedRow { path: KeyPath, source: Source, value: serde_json::Value }`
   is `#[non_exhaustive]` and `Serialize`. `value` is the typed JSON value,
   not a string. A path no layer set is absent; it is never reported as
   `Default`.
7. `Layers::sensitive(path)` marks a key path. In `rows()`, the value of that
   path and of every path under it MUST be the string `"<redacted>"`, while
   `source` is still reported. Redaction MUST NOT change the value
   deserialised into `T`.
8. `rows()` MUST NOT fail and MUST NOT silently drop rows.

### Rationale

Provenance answers "where did this setting come from", which every consumer
with more than one layer eventually needs (`config show` in sc-agent, a
daemon's start-up log). A closed source enum and string-typed provenance
lock that answer to sc-agent's three sources. Typed, serialisable rows let
an agent consume `config show --json` without reparsing. Redaction is needed
because the first consumer to show provenance will show a token.

### Success Criteria

1. A test implements a source with `Source::custom("keychain", "login")`
   and asserts its leaves report that source.
2. Stack `[file {"a":{"b":1,"c":2}}, env P__A__B=3]`: rows are
   `a.b → env P__A__B, 3` and `a.c → file, 2`.
3. Stack `[{"a":{"b":1}}, {"a":5}]`: rows contain only `a`, not `a.b`.
4. A key `{"x.y":1}` produces path `"x.y"` in `Display`, and
   `KeyPath::parse_dotted` of that output equals the original path.
5. `{"a":[1,2]}` yields one row `a` with value `[1,2]`; `{"e":{}}` yields one
   row `e` with value `{}`.
6. With `.sensitive("auth")`, rows for `auth.token` show `"<redacted>"` and
   the loaded `T` still holds the token.
7. `serde_json::to_value(rows)` succeeds and each row's `source` serialises
   with a `kind` and a `name` field.
8. A `T` whose `Deserialize` counts invocations is called exactly once by
   `load_with_provenance` for a three-layer stack.

---

## REQ-CFG-0008: `Loader` is a preset stack on the layered engine

**Status:** Proposed  

### Requirement Statement

This item covers how `Loader` and the free function `load`
([REQ-CFG-0003](requirements.md)) relate to `Layers`
([REQ-CFG-0005](requirements.md)).

1. `Loader::load::<T>()` MUST be implemented as `Loader::layers().load::<T>()`.
2. `Loader::layers(&self) -> Layers` MUST be public and MUST return this
   stack, lowest first: `FileSource` of `<dir>/default.json` (required),
   `FileSource` of `<dir>/local.json` (optional), `EnvSource` with
   `AppName::env_prefix` and the loader's environment, default settings.
3. A consumer MAY push further sources onto the returned `Layers`, for example
   a `ValueSource` of parsed flags, to extend the preset without rebuilding
   it.
4. The crate MUST contain exactly one implementation of file reading, of
   environment-override application and of `merge`. `Loader` MUST NOT keep a
   private pipeline.
5. Every behaviour required by [REQ-CFG-0001](requirements.md) through
   [REQ-CFG-0004](requirements.md) is unchanged, with the error mapping of
   [REQ-CFG-0009](requirements.md) item 3.

### Rationale

Re-expressing `Loader` on the engine removes the PR #153 fork (two env
orders, two sets of file checks) and proves the engine can express the
crate's own conventions. Exposing `layers()` gives sc-agent and the
generated-project template one starting point to extend.

### Success Criteria

1. Every existing test in `crates/sc-config/tests` passes unchanged (only
   error-variant names updated where item 3 of REQ-CFG-0009 maps them).
2. A test compares `Loader::load::<Value>()` with
   `Loader::layers().load::<Value>()` over the REQ-CFG-0001 worked example and
   asserts equality.
3. A test pushes a `ValueSource` onto `Loader::layers()` and asserts it
   overrides an environment variable that sets the same key.
4. `grep -rn "fn merge\|from_slice" crates/sc-config/src` shows one `merge`
   definition and one `from_slice` call site.

---

## REQ-CFG-0009: One `ConfigError` covers every source, with codes

**Status:** Proposed  

### Requirement Statement

This item extends `ConfigError` ([REQ-CFG-0004](requirements.md),
[ADR-CFG-0001](architecture.md)) to the layered engine. There is no second
error type.

1. Every public function of the engine and of every built-in source MUST
   return `ConfigError`. The crate MUST NOT export another error enum.
2. `ConfigError` MUST be `#[non_exhaustive]`.
3. `MissingDefault { path }` is generalised to `MissingFile { path }`, code
   `CFG.MISSING_FILE`. `Loader` reports a missing `default.json` through it.
4. New variants, each with `code()` and `recovery()` like every existing
   variant:

   | Variant | Code | Carries | Recovery |
   |---|---|---|---|
   | `TooLarge { path, limit }` | `CFG.TOO_LARGE` | path, byte limit | Shrink the file or raise the limit the program set. |
   | `ParseFormat { path, message, line, column }` | `CFG.PARSE_FORMAT` | path, parser message, optional line and column | Fix the file at the reported place. |
   | `Source { source, message }` | `CFG.SOURCE` | the `Source` that failed, its message | Follow the message from that source. |

5. `Source { .. }` MUST be constructible by external `LayerSource`
   implementations through `ConfigError::source_failed(source, message)`.
6. `Deserialize` MUST additionally carry `sources: Vec<Source>`: every source
   that contributed a value, in precedence order. Its `files` and
   `variables` fields remain.
7. Every variant's `Display` MUST name the path, variable or source it
   carries.

### Rationale

PR #153's `LayersError` had no codes, no recovery text and free-form
messages, breaking the actionable-error contract of REQ-CFG-0004 for the new
path. One error type keeps one contract. `#[non_exhaustive]` is the only way
to add the next failure mode without breaking every caller, which ADR-CFG-0001
already lists as a cost.

### Success Criteria

1. `grep -rn "pub enum .*Error" crates/sc-config/src` matches only
   `ConfigError`.
2. A test outside the crate matches on `ConfigError` with a wildcard arm and
   compiles; the crate's own source has `#[non_exhaustive]` on the enum.
3. For each new variant a test triggers it and asserts `code()`, a non-empty
   `recovery()` and that `to_string()` contains the carried path or source
   name.
4. A custom source returning `ConfigError::source_failed(Source::custom("k",
   "n"), "denied")` surfaces as `Source` with that source and message.
5. A deserialise failure over a three-source stack carries all three in
   `sources`, in order.

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

1. Every public enum MUST be `#[non_exhaustive]`.
2. Every public struct with public fields MUST be `#[non_exhaustive]` and
   constructible from outside the crate through a constructor or builder.
3. Configuration of a source or stack MUST be by builder methods with
   defaults, not by a struct literal the caller fills in.
4. No public field or argument may require `&'static str` where a runtime
   value is a legitimate input (key paths, variable names, prefixes).
5. `boundaries/sc-config/sc-config.toml` MUST list every public item.

### Rationale

PR #153's `Inputs`, `Key`, `Layered` and `Resolved` were exhaustive structs
with public fields, so the next field any consumer needs breaks every other
consumer. `Key.path: &'static str` stopped keys being built from data.

### Success Criteria

1. A script lists every `pub enum` and `pub struct` with `pub` fields in
   `crates/sc-config/src` and fails if the preceding attributes lack
   `#[non_exhaustive]`.
2. A test crate outside `sc-config` constructs each public struct without a
   struct literal.
3. `grep -rn "&'static str" crates/sc-config/src` returns no match in a public
   signature.
4. `sc-lint lint sc-boundary` passes with the manifest naming every item
   reported by `cargo public-api -p sc-config` (or `rustdoc` JSON when that
   tool is absent).
