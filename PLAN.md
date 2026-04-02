# maski — Interactive TUI for mask

Interactive fuzzy-search interface for [mask](https://github.com/jacobdeichert/mask) taskfiles.

## Pourquoi

La PR [#145](https://github.com/jacobdeichert/mask/pull/145) a été refusée par le mainteneur
qui préfère garder mask minimaliste. `maski` est un wrapper externe qui utilise
`mask --introspect` pour récupérer l'AST JSON du maskfile, sans aucun couplage interne.

## Fonctionnalités

- **Fuzzy search** des commandes via skim
- **Navigation hiérarchique** dans les sous-commandes (→ entrer, ← retour, Esc quitter)
- **Preview panel** : description, args, flags, script source
- **Prompts interactifs** pour les arguments requis/optionnels et flags (dialoguer)
- **Exécution** via subprocess `mask <command> <args>`

## Architecture

```
maski/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI : appelle mask --introspect, parse JSON, lance TUI
│   ├── types.rs          # Structs serde pour désérialiser le JSON --introspect
│   └── interactive.rs    # TUI skim + dialoguer + exécution via subprocess mask
├── README.md
└── LICENSE
```

### Comment ça marche

```
maski                          mask --introspect
  │                                  │
  │  1. lance ───────────────────▶   │
  │                                  │
  │  2. reçoit JSON ◀───────────── { commands: [...] }
  │
  │  3. TUI skim (fuzzy search + navigation + preview)
  │
  │  4. dialoguer prompts (args, flags)
  │
  │  5. subprocess ──────────────▶  mask <cmd> --flag val arg1
```

### Format JSON de `mask --introspect`

```json
{
  "title": "...",
  "description": "...",
  "commands": [
    {
      "name": "build",
      "description": "Build the project",
      "level": 2,
      "script": { "executor": "bash", "source": "cargo build\n" },
      "subcommands": [],
      "required_args": [{ "name": "target" }],
      "optional_args": [{ "name": "profile" }],
      "named_flags": [{
        "name": "release", "short": "r", "long": "release",
        "description": "Release mode", "takes_value": false,
        "choices": [], "required": false, "validate_as_number": false
      }]
    }
  ]
}
```

### Dépendances

| Crate | Usage |
|-------|-------|
| `skim` 0.10 | Fuzzy finder TUI avec preview |
| `dialoguer` 0.11 | Prompts interactifs (input, select, confirm) |
| `serde` + `serde_json` | Désérialiser le JSON de --introspect |
| `colored` 2 | Couleurs terminal |
| `clap` 4 | CLI args (--maskfile, --help, --version) |

**Pas de dépendance sur `mask-parser`** — tout passe par le JSON.

### Différences avec le fork

| | Fork mask (PR #145) | maski |
|---|---|---|
| Couplage | Dépend du code interne mask | Zéro — juste le JSON |
| Installation | Remplace mask | Coexiste avec mask |
| Compat futures versions | Doit suivre les releases | Stable tant que --introspect existe |
| Executor | Interne (copie de mask) | Subprocess `mask` |
| Dépendances | mask-parser, skim, dialoguer | serde_json, skim, dialoguer |

### Prérequis

`mask` doit être installé et dans le PATH (pour `--introspect` et l'exécution).

## CLI

```
maski                       # lance le TUI dans le répertoire courant
maski --maskfile ./ops.md   # utilise un maskfile spécifique
```

## Exécution des commandes

Quand l'utilisateur sélectionne une commande et remplit les prompts :

```bash
# Commande simple
mask build

# Avec args positionnels
mask deploy production

# Avec flags
mask build --release --target x86_64

# Sous-commande
mask db migrate --seed
```

Les arguments et flags sont passés tels quels à `mask` en subprocess.

## Étapes

1. `cargo init maski` + Cargo.toml
2. `types.rs` : structs serde miroir du JSON --introspect
3. `interactive.rs` : adapter depuis le fork (remplacer executor par subprocess mask)
4. `main.rs` : arg parsing + `mask --introspect` + lancer TUI
5. `cargo build && cargo test`
6. Tester sur des maskfiles existants (avec sous-commandes)
7. README avec démo GIF
8. Créer repo GitHub + `cargo publish`

## Questions ouvertes

- `--no-execute` flag pour juste afficher la commande construite sans l'exécuter ?
- Supporter un mode où mask n'est pas installé (fallback sur mask-parser) ?
