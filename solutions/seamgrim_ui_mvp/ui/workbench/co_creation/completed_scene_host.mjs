import { mountCompletedSceneView } from './completed_scene_view.mjs';

// Host lifetime only: observations stay owned by the captured run supplier.
export function installCompletedSceneHost(root, { selectionRoot, selection,
  canonicalForFrame, semanticForFrame, selectOutput, renderSpace2d }) {
  let ready = false, disposed = false;
  const view = mountCompletedSceneView(root, {
    readPair: () => ready && !disposed ? selection.selected() : null,
    canonicalForFrame, semanticForFrame, selectOutput, renderSpace2d,
  });
  const clear = () => { ready = false; return view.refresh(); };
  selectionRoot.addEventListener('change', clear);
  return {
    list() { if (disposed) return []; clear(); return selection.list(); },
    choose(a, b) {
      if (disposed) throw Error('완료 실행 표시가 종료됐습니다.');
      clear(); // A failed new choice must not leave the old pair displayed.
      const pair = selection.choose(a, b);
      ready = true; view.refresh(); return pair;
    },
    clear,
    dispose() {
      if (disposed) return;
      disposed = true; ready = false;
      selectionRoot.removeEventListener('change', clear); view.dispose();
    },
  };
}
