import { markdownToHtml } from "./markdown.js";

export class OverlayDescription {
  constructor(el) {
    this.el = el;
    this.visible = false;
  }

  setContent(markdown) {
    if (!this.el) return;
    this.el.innerHTML = markdownToHtml(markdown);
  }

  toggle() {
    this.visible = !this.visible;
    this.sync();
    return this.visible;
  }

  show() {
    this.visible = true;
    this.sync();
  }

  hide() {
    this.visible = false;
    this.sync();
  }

  sync() {
    if (!this.el) return;
    this.el.classList.toggle("hidden", !this.visible);
  }
}
