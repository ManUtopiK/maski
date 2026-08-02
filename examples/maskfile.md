# Example Tasks

> A sample maskfile showing off what `maski` renders in the preview panel.
> Every task here is harmless — they only `echo`.

Run it with:

```sh
maski --maskfile examples/maskfile.md
```

## build (target)

> Build the project for a given target.

Takes one **required** argument, so `maski` prompts for it before running.

**OPTIONS**

- release
  - flags: -r --release
  - type: boolean
  - desc: Optimized build instead of the debug profile.
- jobs
  - flags: -j --jobs
  - type: string
  - desc: Number of parallel jobs.

::note
Boolean flags reach the script as the string `"true"` when set, and are empty otherwise.
::

```sh
echo "building $target (release=${release:-false}, jobs=${jobs:-auto})"
```

## test [filter]

> Run the test suite, optionally filtered by name.

The argument is in `[brackets]`, so it is **optional** — leave the prompt empty to skip it.

- [x] unit tests
- [x] integration tests
- [ ] end-to-end tests

```sh
echo "running tests ${filter:+matching '$filter'}"
```

## db

> Database chores. This heading has no script of its own, only subcommands.

Because it groups subcommands, `maski db` opens the TUI right here instead of running anything.

### migrate

> Apply pending migrations.

**OPTIONS**

- steps
  - flags: -n --steps
  - type: string
  - desc: How many migrations to apply. Defaults to all of them.

| Direction | Command      | Reversible |
| --------- | ------------ | ---------- |
| Up        | `db migrate` | yes        |
| Down      | `db rollback`| yes        |
| Reset     | `db reset`   | **no**     |

```sh
echo "applying ${steps:-all} migration(s)"
```

### seed

> Load fixture data into the current database.

:::tip Idempotent
Seeding twice is safe — records are upserted, never duplicated.
:::

```sh
echo "seeding fixtures"
```

### reset

> Drop everything and rebuild from scratch.

> [!CAUTION]
> This destroys local data. There is no undo.

::details What gets dropped
Tables, sequences, and the migration history table.
::

```sh
echo "resetting database"
```

## deploy

> Ship the app. Also a pure group of subcommands.

### staging

> Deploy to the staging environment.

> [!NOTE]
> Staging redeploys on every push to `main`, so this is rarely needed by hand.

```sh
echo "deploying to staging"
```

### production

> Deploy to production.

**OPTIONS**

- version
  - flags: -v --version
  - type: string
  - desc: Tag to deploy. Defaults to the latest release.
- confirm
  - flags: --confirm
  - type: boolean
  - desc: Skip the interactive confirmation.

> [!WARNING]
> Production deploys are announced in `#releases`. Check the dashboard first.

Deploy order:

1. Build the image
2. Run migrations
3. Roll out :badge[blue/green]

```sh
echo "deploying ${version:-latest} to production"
```

## docs

> Documentation site.

### dev

> Start the docs dev server with live reload.

Served on ~~localhost:3000~~ **localhost:4000** since the last upgrade.

```sh
echo "docs dev server on http://localhost:4000"
```

### build

> Build the static docs site.

Note that this `build` and the top-level `build` are different tasks — the breadcrumb in the header tells them apart.

```sh
echo "building static docs"
```
