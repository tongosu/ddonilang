import { buildControlSpecsFromDdn } from "./control_parser.js";

export class SliderPanel {
  constructor({ container, statusEl, onCommit } = {}) {
    this.container = container;
    this.statusEl = statusEl;
    this.onCommit = typeof onCommit === "function" ? onCommit : () => {};
    this.specs = [];
    this.values = {};
    this.axisKeys = [];
    this.defaultAxisKey = "";
    this.defaultXAxisKey = "";
  }

  parseFromDdn(ddnText, { preserveValues = false } = {}) {
    const parsed = buildControlSpecsFromDdn(ddnText);
    const nextSpecs = Array.isArray(parsed.specs) ? parsed.specs : [];
    const nextValues = {};
    nextSpecs.forEach((spec) => {
      const prev = Number(this.values?.[spec.name]);
      if (preserveValues && Number.isFinite(prev)) {
        nextValues[spec.name] = prev;
        return;
      }
      nextValues[spec.name] = Number(spec.value);
    });

    this.specs = nextSpecs;
    this.values = nextValues;
    this.axisKeys = Array.isArray(parsed.axisKeys) ? parsed.axisKeys : [];
    this.defaultAxisKey = String(parsed.defaultAxisKey ?? "").trim();
    this.defaultXAxisKey = String(parsed.defaultXAxisKey ?? "").trim();

    if (this.statusEl) {
      if (parsed.source === "prep") {
        this.statusEl.textContent = `control 채비: ${this.specs.length}개`;
      } else if (parsed.source === "meta") {
        this.statusEl.textContent = `control meta: ${this.specs.length}개`;
      } else {
        this.statusEl.textContent = "control: -";
      }
    }
    this.render();
    return {
      specs: [...this.specs],
      values: { ...this.values },
      axisKeys: [...this.axisKeys],
      defaultAxisKey: this.defaultAxisKey,
      defaultXAxisKey: this.defaultXAxisKey,
      source: parsed.source,
    };
  }

  render() {
    if (!this.container) return;
    this.container.innerHTML = "";

    if (!this.specs.length) {
      const empty = document.createElement("div");
      empty.className = "hint";
      empty.textContent = "조절 가능한 채비 상수가 없습니다.";
      this.container.appendChild(empty);
      return;
    }

    this.specs.forEach((spec) => {
      const row = document.createElement("div");
      row.className = "slider-row";

      const label = document.createElement("div");
      label.className = "slider-label";
      label.textContent = `${spec.name}${spec.unit ? ` (${spec.unit})` : ""}`;

      const range = document.createElement("input");
      range.type = "range";
      range.min = String(spec.min);
      range.max = String(spec.max);
      range.step = String(spec.step);
      range.value = String(this.values[spec.name]);

      const number = document.createElement("input");
      number.type = "number";
      number.min = String(spec.min);
      number.max = String(spec.max);
      number.step = String(spec.step);
      number.value = String(this.values[spec.name]);

      const updateLocal = (nextValue) => {
        const numeric = Number(nextValue);
        if (!Number.isFinite(numeric)) return;
        this.values[spec.name] = numeric;
      };

      const commit = (nextValue) => {
        updateLocal(nextValue);
        this.onCommit(this.getValues());
      };

      range.addEventListener("input", () => {
        number.value = range.value;
        updateLocal(range.value);
      });
      range.addEventListener("change", () => {
        commit(range.value);
      });

      number.addEventListener("input", () => {
        range.value = number.value;
        updateLocal(number.value);
      });
      number.addEventListener("change", () => {
        commit(number.value);
      });

      row.appendChild(label);
      row.appendChild(range);
      row.appendChild(number);
      this.container.appendChild(row);
    });
  }

  getValues() {
    return { ...this.values };
  }

  setValues(nextValues = {}) {
    this.values = { ...this.values, ...nextValues };
    this.render();
  }

  getAxisKeys() {
    return [...this.axisKeys];
  }

  getDefaultAxisKey() {
    return this.defaultAxisKey;
  }

  getDefaultXAxisKey() {
    return this.defaultXAxisKey;
  }

  getSpecs() {
    return [...this.specs];
  }
}
