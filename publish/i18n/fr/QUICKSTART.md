# Démarrage rapide (Français)

> Traduction starter. Les commandes et noms de fichiers restent canonical.

## 1. Construire depuis les sources

Pré-requis : Rust + Cargo

~~~sh
cargo build --release
~~~

Vérifier la CLI :

~~~sh
cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- --help
~~~

## 2. Lancer le workspace Seamgrim

Démarrer le serveur local :

~~~sh
python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py
~~~

Ouvrir dans le navigateur :

~~~txt
http://localhost:8787/
~~~

Le workspace peut ouvrir cet inventaire d'exemples 'solutions/seamgrim_ui_mvp/samples/index.json'.

## 3. Contrôles de régression produit

~~~sh
python tests/run_seamgrim_product_stabilization_smoke_check.py
node tests/seamgrim_sample_grid_space_runner.mjs
node tests/seamgrim_studio_layout_contract_runner.mjs
node tests/seamgrim_run_toolbar_compact_runner.mjs
python tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py
~~~

## 4. Chemin de release binaire

Quand des binaires sont publiés, les télécharger depuis GitHub Releases. Les binaires ne sont pas stockés dans le dépôt git.

- Windows: '.\ddonirang-tool.exe --help'
- macOS/Linux : d'abord 'chmod +x ./ddonirang-tool', puis './ddonirang-tool --help'
