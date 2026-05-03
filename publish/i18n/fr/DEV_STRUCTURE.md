# Structure de développement (Français)

> Traduction starter. Les commandes et noms de fichiers restent canonical.

Ceci est un résumé public localisé. Le fichier canonical détaillé est '../../DDONIRANG_DEV_STRUCTURE.md'.

## Couches principales

| Couche | Chemin | Rôle |
| --- | --- | --- |
| core | 'core/' | coeur de moteur déterministe |
| lang | 'lang/' | grammaire, parser, canonicalization |
| tool | 'tool/' | implémentation runtime/tool |
| CLI | 'tools/teul-cli/' | exécution CLI et checks |
| packs | 'pack/' | pack evidence exécutable |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | workspace web et vues Bogae |
| tests | 'tests/' | tests d'intégration et produit |
| publish | 'publish/' | documents publics |

## Seamgrim workspace V2

- 'ui/index.html': point d'entrée unique
- 'ui/screens/run.js': écran d'exécution et exécution current-line
- 'ui/components/bogae.js': rendu Bogae console/graph/space2d/grid
- 'ui/seamgrim_runtime_state.js': madi, état runtime et résumé mirror
- 'tools/ddn_exec_server.py': serveur statique local et API auxiliaire

## Principe runtime

- Le DDN runtime, les packs, state hashes et enregistrements mirror/replay possèdent la truth.
- Bogae est une view layer et ne possède pas la runtime truth.
- Python/JS peuvent orchestrer les checks et l'UI, mais ne doivent pas remplacer la sémantique par du test-only lowering.

## Evidence actuelle

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
