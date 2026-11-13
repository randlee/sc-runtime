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

This item covers the file layer of the `sc-config` crate (`crates/sc-config`).
The design fixes the two file names and that `local.json` is gitignored. The
merge rules below are decided in this document.

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

4. The result is then built from `config/default.json` alone.
7. Those overrides take precedence over both files ([REQ-CFG-0002](requirements.md)).
9. It selects only the environment-variable prefix.

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

**MUST statements:**

- The crate MUST export this function:
- `load` MUST read `config/default.json`, resolved against the process's current working directory.
- `config/default.json` MUST exist.
- If it does not, `load` MUST return the `ConfigError` variant for a missing default file ([REQ-CFG-0004](requirements.md)).
- `load` MUST read `config/local.json` from the same directory when that file exists.
- Each file MUST be read as bytes.
- Each file's bytes MUST be parsed as one JSON document into a `serde_json::Value` (`serde_json::from_slice`).
- `load` MUST merge the `local.json` value over the `default.json` value with the function `merge(base, overlay)` defined in the table below. (see table above)
- `load` MUST then apply the environment-variable overrides of app `app` to the merged value.
- `load` MUST use `serde_json::from_value::<T>` for its exactly-once deserialisation of the final value into `T`.

**MUST NOT statements:**

- An absent `config/local.json` MUST NOT be an error.
- The `app` argument MUST NOT change which files are read.
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
   files are read (entry 9).

---

## REQ-CFG-0002: Environment overrides named `APP__KEY__KEY`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: Silent restructuring means changing the document shape implicitly during the override key-path walk instead of applying the stated replacement or error rule. The prefix is the app name with ASCII lower-case letters upper-cased and `-` replaced by `_`, followed by `__`. A segment is one piece of the variable name after that prefix, split by `str::split("__")`. The key path `k1, k2, ..., kn` is the sequence of segments converted to ASCII lower case. The key-path walk starts at the document root and moves through keys `k1` to `k(n-1)` in order. The override value is the parsed JSON value, or the unchanged value as a JSON string when parsing fails.

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

- The prefix MUST be the app name with every ASCII lower-case letter upper-cased and every `-` replaced by `_`, followed by `__` (two underscores).
- A variable MUST be treated as an override if and only if its encoded name starts with that prefix.
- Every variable whose name does not start with the prefix MUST be ignored, whatever its value.
- The part of the variable name after the prefix MUST be split into segments exactly as Rust `str::split("__")` splits it.
- Each resulting segment MUST be converted to ASCII lower case.
- The variable's value MUST be parsed with `serde_json::from_str::<serde_json::Value>`.
- If parsing succeeds, the parsed JSON value MUST be used.
- If parsing fails, the value MUST be used unchanged as a JSON string.
- Starting at the document root, for each of `k1` to `k(n-1)`: if the current value is an object that lacks the key, an empty object MUST be created at that key.
- If the current value at any step of that walk, including the root, is not an object (it is a string, number, boolean, array or `null`), the load MUST fail with the `ConfigError` variant for an override that cannot be applied ([REQ-CFG-0004](requirements.md)).
- At the last object in the key-path walk, key `kn` MUST be set to the override value.
- Overrides MUST take precedence over `config/default.json` and `config/local.json`.

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

Definitions: JSON means JavaScript Object Notation. API means application programming interface. A temporary-directory API allocates a distinct test directory rather than using a literal path. The convenience form supplies the app name, current-directory-relative `config` path and process environment to the loader. `merge(base, overlay)` recursively merges object keys and replaces other values with the overlay, including arrays and null, as specified in REQ-CFG-0001. Environment-override rules in REQ-CFG-0002 select matching app-prefixed variables, decode their key paths and values, and apply them to the merged document.

This item covers the `Loader` type of the `sc-config` crate
(`crates/sc-config`).

The caller may construct the environment argument entirely in memory as a vector of variable-name/value pairs.
10. No code in the crate may call `std::env::set_var`,
    `std::env::remove_var` or `std::env::set_current_dir`, in `src/` or in
    tests. Setting the environment or the working directory of a child
    process with `std::process::Command::env` and
    `std::process::Command::current_dir` is allowed, because it does not
    change the state of the test process.
11. No file under `crates/sc-config`, code or tests, may contain a literal temporary-directory path or a literal home-directory path (`/tmp/`, `/home/`, `/Users/`). The repository-wide rule is [NFR-RUN-0005](../requirements.md): non-test code contains no such literal, and every test that writes files uses its own temporary directory. Extending the literal-path ban to this crate's test code is decided in this document.

Using `./config` and the process environment as the defaults of `load` is
decided in this document; the design fixes only the file names
`config/default.json` and `config/local.json`. The call takes
`AppName::new("my-app")`.

Decided: `Loader::new(app: AppName, config_dir: PathBuf, environment: Vec<(OsString, OsString)>) -> Loader`.
The constructor stores those arguments and does not return `Result`.
`Loader::load` and `load` return `Result<T, ConfigError>`. Environment pairs
are `OsString` so a non-Unicode value is reported instead of panicking.

**MUST statements:**

- The crate MUST export a public type `Loader`.
- A `Loader` MUST be given the app name explicitly by the caller.
- A `Loader` MUST be given the config directory (a filesystem path) explicitly by the caller.
- `Loader::new` MUST require the caller to provide the `environment` argument.
- The `environment` argument MUST have type `Vec<(OsString, OsString)>`.
- The entries of `environment` MUST be iterated as `(variable name, value)` pairs.
- `Loader` MUST expose the public generic method `Loader::load::<T>()`.
- For every `T: serde::de::DeserializeOwned`, `Loader::load::<T>()` MUST be callable.
- The return type of `Loader::load::<T>()` MUST be `Result<T, ConfigError>`.
- `Loader::load` MUST read `default.json` only from the given config directory.
- `Loader::load` MUST read `local.json` only from the given config directory.
- `Loader::load` MUST take overrides only from the given environment source.
- `Loader::load` MUST merge the values from its supplied configuration directory using the `merge(base, overlay)` rules in [REQ-CFG-0001](requirements.md).
- `Loader::load` MUST apply its supplied environment pairs to the merged document using the override rules in [REQ-CFG-0002](requirements.md).
- The free function `load::<T>(app)` MUST provide the `Loader::load` operation with default inputs.
- The behavior of `load::<T>(app)` MUST equal the behavior of a `Loader` supplied with `app`, the path `config` resolved relative to the current working directory, and the process-environment pairs.
- When reading the process environment, `load` MUST use `std::env::vars_os`.
- Each `sc-config` test MUST obtain its test directories from a temporary-directory API that allocates a distinct directory for that test.

**MUST NOT statements:**

- `Loader` MUST NOT access the process environment.
- The free function `load` MUST NOT use `std::env::vars`.

### Rationale

`std::env::vars` can panic when an environment variable is not valid Unicode. The convenience free function uses `std::env::vars_os` so environment pairs retain their `OsString` representation and non-Unicode input is reported instead of causing that panic.

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
   with `.current_dir(D)`, `.env_clear()` and `.env("MY_APP__A", "1")`. The helper calls
   `sc_config::load::<serde_json::Value>` for `AppName::new("my-app")` and prints the result as
   JSON on standard output. The test asserts the printed value is `{"a":1}`.
   How the helper executable is built (for example a test-only binary target
   of the crate) is left to the implementation.
6. An integration test covers entry 6. It starts a helper executable with a cleared child environment and
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
    fixed temporary or home path in code or tests (entry 11).

---

## REQ-CFG-0004: `ConfigError` variants name the file or variable at fault

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: The failure table is the table of five load failures in this requirement. Failure N is row N of the failure table. A failure's data is the entry in that row's third column. The five failure variants are `MissingDefault`, `Read`, `Parse`, `Override` and `Deserialize`, for failures 1 to 5 in that order.

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
- Each of the five failure variants MUST carry its failure's data as typed fields of the variant.
- A `local.json` that exists but cannot be read MUST produce failure 2.
- Failure 1 MUST produce the variant `MissingDefault`.
- Every I/O error on `default.json` or `local.json` other than failure 1 MUST produce the variant `Read`.
- `ConfigError` MUST implement `std::fmt::Debug`.
- `ConfigError` MUST implement `std::fmt::Display`.
- `ConfigError` MUST implement `std::error::Error`.
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

This item binds all code under `crates/sc-config/src`.

The panic rule for the four library crates of this repository is owned by [NFR-RUN-0009](../requirements.md). There is no allowance for a commented exception. Test code is exempt. This item applies that rule to `sc-config` and adds the crate-specific entries 5 and 6.

1. In Rust this is the discriminated union the design calls for: the caller receives either the value or a typed error and must handle both.
2. No public function or method may return `Option` to signal failure, return
   a boxed or opaque error (`Box<dyn Error>`, `anyhow::Error`), or report
   failure by printing or logging.
3. No code path reachable from a public function or method may panic, for any
   input: any app name, any directory path, any file content, any environment
   content including variables that are not valid Unicode.
7. Implementations of standard traits on public types (`Display`, `Debug`,
   `std::error::Error`, `From`) are not "public methods" for entry 1, but
   entries 3, 4 and 5 apply to them.
8. Code inside `#[cfg(test)]` modules and under `crates/sc-config/tests/` is
   exempt from entries 4 and 5.

Decided: Only an operation that can fail returns `Result<T, ConfigError>`.
`Loader::new` stores a validated `AppName` plus the caller's directory and
environment, so it returns `Loader`. `AppName::new` can fail and returns
`Result<AppName, ConfigError>`. [NFR-RUN-0009](../requirements.md) requires
`Result` of operations that can fail.

**MUST statements:**

- Every public function and public method of `sc-config` that can fail MUST return `Result<T, ConfigError>`, where `ConfigError` is the crate's public typed error enum ([REQ-CFG-0004](requirements.md)).
- Where a slice, string, `Vec` or `serde_json::Value` is accessed, the code MUST use `get`, pattern matching or iterators.
- It MUST use `std::env::vars_os()`.

**MUST NOT statements:**

- That rule is: code reachable from a public function MUST NOT use `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, or panicking `[]` indexing.
- Code reachable from a public function MUST NOT use `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, or panicking `[]` indexing ([NFR-RUN-0009](../requirements.md)).
- Stricter than the repository rule, decided in this document: code reachable from a public function MUST NOT use `assert!`.

- Stricter than the repository rule, decided in this document: code reachable from a public function MUST NOT use `assert_eq!`.

- Stricter than the repository rule, decided in this document: code reachable from a public function MUST NOT use `assert_ne!`.
- Stricter than the repository rule, decided in this document: non-test code MUST NOT call `std::env::vars()`, which panics on a variable that is not valid Unicode.
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
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

For the Phase D deliverable, this item binds `crates/sc-config/Cargo.toml` and owns its third-party dependency list. A later async or reload option is not forbidden and creates no work in Phase D (PRD-ENG-5/6). The edges between the crates of
this workspace, the forbidden edges, and the boundary manifest under
`boundaries/sc-config/` that encodes them are owned by
[REQ-RUN-0005](../requirements.md), which refers to this item for the
third-party list.

4. For Phase D, the forbidden edges of `sc-config` are `sc-observability`,
   `sc-observability-otlp` (the OTel export crate of `sc-observability`),
   `tokio`, `sc-transport`, `sc-command` and `sc-runtime`. The obligation,
   and the `forbidden_edges` entries of the boundary manifest that
   `sc-lint-boundary` checks, are stated in
   [REQ-RUN-0005](../requirements.md). This item adds only the `cargo tree`
   check in criterion 3.

For Phase D, `[dev-dependencies]` is empty. Tests create a private directory
under `std::env::temp_dir` and remove it when the test ends. No third test
crate is added in Phase D.

**MUST statements:**

- For Phase D, the `[dependencies]` table of `crates/sc-config/Cargo.toml` MUST contain exactly two entries: `serde` and `serde_json`.
- The `ConfigError` type MUST be written by hand.

**MUST NOT statements:**

- For Phase D, `[dependencies]` MUST NOT contain optional dependencies.
- For Phase D, `[dependencies]` MUST NOT contain target-specific dependency tables that add a third crate.
- `thiserror` MUST NOT be added.
- `anyhow` MUST NOT be added.

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
   the keys `serde` and `serde_json` and no others, and the file has no
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
