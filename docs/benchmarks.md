# Benchmarks

Four suites, all `#[ignore]`d so an ordinary `cargo test` never runs them.

| task | what it measures | needs a browser |
|---|---|---|
| `task ssr-bench` | server-side rendering, per phase | no |
| `task hydration-bench` | hydration of server markup | yes |
| `task reactive-bench` | the reactive graph alone | yes |
| `task dom-bench` | the whole framework: graph, DOM commands, rendering | yes |

`task bench` runs all four in that order; `task bench-compare` runs all four against the last
run of each. The browser suites need a WebDriver on `localhost:9515` (`chromedriver
--port=9515`).

Every suite prints a table, writes a JSON run under `target/bench/<suite>/`, and can print a
delta table against an earlier run. Timings are printed and never asserted - they swing
severalfold between machines. What is asserted is the deterministic part: command counts for
SSR and for the DOM suite, the graph's cutoff and fan-out invariants for the reactive one, and
hydration coverage plus DOM mutation counts for hydration.

The two browser suites that run workloads rather than page loads take `VERTIGO_BENCH_SCALE`,
which multiplies every workload's iteration count - lower it to iterate quickly, raise it on a
machine fast enough to make the batches too short to time.

## One shape for all four

`tests/bench-report/` holds the table, the file and the comparison. A suite supplies a `Suite`
descriptor - its name, its metric columns and their units, its counters, and how much movement
counts as noise - and the rest is shared. So a run of any suite reads the same way:

- **metrics** are timings, each with a unit and an absolute noise floor. A delta has to clear
  both that floor and the suite's relative band before it prints as anything but `~`, because
  the percentage alone turns a one-microsecond scheduler hiccup on a three-microsecond phase
  into a 40% "regression". One metric is the *headline*: it gets the summary table and the
  stability check.
- **counters** are deterministic - command counts, node counts, coverage, checksums. They are
  compared exactly, and a change in one is reported **before** the timings, because a row that
  got slower may simply have been asked to do more.
- **artifacts** are what the run shipped, and are per run rather than per row. See below.

A run's file records the machine, the toolchain and the harness settings. When a baseline
disagrees with the current run about any of those, the comparison says so above the table:
what it is about to print is then a comparison of two machines rather than of two commits.

Files written before the four suites were unified (schema 1) are still readable; the
comparison says when it has fallen back to that adapter.

## Artifact sizes

Every suite records the size of everything it built, above the timings:

```text
  artifacts
    demo wasm                   999 442   gz     313 200  code 936 681  data 58 850  elem 1 852  function 1 581  other 440
    demo js                      30 568   gz      10 024
```

- The **wasm section breakdown** is read out of the module itself, so a size regression is
  attributable without reaching for `wasm-objdump`: compiled code and embedded data move for
  different reasons and are worth telling apart. Sizes are section payloads, which is the
  convention `wasm-objdump -h` prints, so they sum to slightly less than the file - the header
  and each section's own id and length byte belong to no section. Sections under a kilobyte are
  summed into `other`; the full map is in the JSON.
- **`gz`** is the transfer size to compare against, at a fixed compression level. It will not
  match `gzip -9` to the byte - a different deflate implementation makes slightly different
  choices - but it is consistent from run to run, which is what a comparison needs. It is an
  indication rather than the exact number a visitor downloads: `vertigo serve` negotiates
  brotli first and at its own compression level, so a real transfer is usually a little
  smaller. The benchmarks themselves run with `--disable-compression`, so that what they time
  is vertigo rather than brotli, and so that they stay comparable with baselines recorded
  before compression existed.
- `wasm_run.js` is the same file for every suite: it comes from the vertigo crate rather than
  from the subject app. Four suites reporting an identical number is a free cross-check.

The comparison prints this **exactly, with no noise band**, because a file is the size it is:

```text
  artifacts (exact - sizes are deterministic, so there is no noise band here)
    demo wasm                   999 442   +42 825 B (+4.5%)   gz +9 104 B (+3.0%)
      code                      936 681   +42 121 B (+4.7%)
      data                       58 850   +672 B (+1.2%)
```

Only what moved gets a line. An artifact whose size is unchanged but whose content hash differs
is reported as `unchanged in size, but rebuilt` - vertigo's filenames are content-addressed, so
that case is detectable, and it is exactly where a size table alone would wrongly read as
"nothing happened".

Nothing here is asserted. Every other deterministic number in these suites is checked against a
constant derived from the thing it measures, and there is no such constant for a size: the only
meaningful statement is a difference from a named earlier run.

## Comparing two commits

```bash
git checkout <base> && task ssr-bench          # writes target/bench/ssr/<sha>-<ts>.json
git checkout <head>
VERTIGO_SSR_BENCH_BASELINE=target/bench/ssr/<base>-<ts>.json task ssr-bench
```

`task ssr-bench-compare` does the same against `target/bench/ssr/latest.json`, and each of the
other three suites has the same pair. Every suite also takes a `..._BENCH_LABEL` variable,
which names the run in the file and in the table; it defaults to the commit, except for
hydration, where it defaults to the branch.

Two things about `latest.json` are worth knowing, because both are deliberate:

- **A run is written before it is checked, but promoted afterwards.** The timestamped
  `<label>-<ts>.json` is written as soon as the measuring is done, whatever happens next - a
  suite that took twenty minutes of page loads should not throw them away because a baseline
  path was wrong. `latest.json` is only pointed at the run once it has passed its own
  assertions, so a run whose command counts the suite calls wrong never becomes the thing the
  next comparison silently subtracts against.
- **A named baseline that does not exist is not an error.** The `-compare` tasks name
  `latest.json` by default, and the first time a suite is ever run there is no earlier run; it
  says so and prints no delta table. A baseline that *is* there and cannot be read - truncated,
  or belonging to another suite - does fail, because comparing against nothing would read as
  "nothing changed".

## Comparing two hydration implementations

`tests/hydration-bench/` exists to answer a question the other suites cannot: hydration was
rewritten from JavaScript into Rust on a branch, and the two implementations share no code.

The suite therefore measures only through three things that predate both and are identical on
both, so that **neither branch needs instrumenting** - instrumentation added separately to two
implementations is a thing the comparison could be an artifact of:

- an inline `<script>` the subject app server-renders into `<head>`, which installs a
  `MutationObserver` before wasm boots;
- `performance.now()`, read by the app at the first and last line of its render and carried
  out on the sentinel element's attributes;
- `window.__vertigo_hydration`, whose five field names are an older contract than either
  implementation.

To run it against a branch that has its own hydration:

```bash
task hydration-bench                                   # on the branch you are holding

git worktree add --detach ../vertigo-other origin/<the-other-branch>
cd ../vertigo-other
git cherry-pick <the hydration-bench commit>           # see the note below
VERTIGO_HYDRATION_BENCH_LABEL=other \
VERTIGO_HYDRATION_BENCH_BASELINE=<absolute path to the first run's json> \
  task hydration-bench
```

**The cherry-pick is not always clean.** The commit only adds `tests/hydration-bench/`, but it
also edits `Cargo.toml`, `tests/Cargo.toml`, `Taskfile.yaml` and `.gitignore` next to lines
that other benchmark suites added - so on a branch that has a different set of suites, those
four files conflict on adjacency alone. The resolution is mechanical: keep the target branch's
version of each file and add the hydration entries (workspace member, dev-dependency,
`[[test]]` target, ignore paths). Basing the benchmark commit directly on the common ancestor
avoids it entirely.

### Reading the table

`render_ms` is the subject app building its own tree. That is the same work whichever
implementation hydrates it, so it is a **control**: if it moves between two runs, something
other than hydration did and the rest of the table should not be read. `hydrate_ms` is
everything the framework does with that tree, and is the number the suite exists to produce.

`mutations` counts operations on nodes **already in the document**, not nodes touched - a
`MutationObserver` reports one record with one added node when a subtree is attached, however
large it is. So the `[off]` rows, which rebuild detached and attach once, are not a mutation
baseline for the `[on]` rows, and nothing in the suite compares them. Between two hydration
implementations the count is exactly comparable: both work on the same attached server nodes.

The demo rows carry coverage only, and print `-` for everything else. The probe is part of the
subject app's markup, and a page's own markup is the only way to get a script in front of wasm
across a real navigation, so `vertigo-demo` has no observer and no marks. Its
`matched`/`hydratable` still come from the framework, and answer the question the controlled
pages cannot: whether a real application hydrates as completely under one implementation as
under the other.

### Two things that will bite whoever edits the pages

- **Nesting deeper than 512 is not measurable.** Chrome's HTML parser caps element nesting at
  512 and flattens the rest, so a server document nested past that is not the document the
  browser holds, and hydration cannot match nodes that were never built. `/deep` reported 516
  of 607 matched at depth 600 and looked like a defect in the matcher; it was the parser.
- **Invalid HTML defeats hydration completely.** `<tr>` written straight inside `<table>` makes
  the parser insert a `<tbody>`, so the parsed server document and the application's tree
  disagree at the first child and nothing below the table matches. That page hydrated 6 of
  6 807 nodes until the `<tbody>` was written explicitly.

## Reading the reactive and end-to-end suites together

`reactive-bench` measures propagation alone: a write enters the graph, dependents recompute,
and nothing reaches the DOM. `dom-bench` measures the same machinery with the rest of the
framework attached - the write becomes DOM commands, crosses into JS and lands in the
document. Run together, the pair says *which half* moved: a change visible in both is in the
graph, a change visible only in `dom-bench` is in command generation, the wire format or the
JS applier.

Both report per workload:

- `batch_ms`, the headline - one batch of `iters` operations, taken three times;
- `per_op_us`, the same divided by `iters`, which is what to quote;
- `runs`, the number of compute closures that actually ran, which is what distinguishes *the
  write was cut off by the equality check* from *the work was optimised away*;
- `checksum`, folded out of the graph and rendered, so that a table cannot be pasted from a run
  whose work LLVM deleted.

`dom-bench` adds `cmds` (DOM commands per operation), `leaked` (tracked DOM nodes left behind
after teardown, which must be zero) and a per-variant breakdown, stored in the JSON under
`cmd.<variant>` and printed in its own block rather than as thirteen mostly-empty columns.
