# Observer Registry

`OBSERVER_REGISTRY.json` lists UI files that are read-only observers.

Add a new observer file here before relying on the D40 guard.

Observer files must not import or call mutation functions such as reset, step, run, restore, set-param, or AI action injection.

Driver files such as `screens/run.js` are intentionally not registered here.
