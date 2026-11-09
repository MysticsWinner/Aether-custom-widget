// Copyright (c) Aether Platform. Licensed under the MIT License.
"use strict";

class BatchRenderCanvas {
  constructor() {
    this._commands = [];
    this._invalidated = [];
  }

  clear(color) {
    this._commands.length = 0;
    this._commands.push({ kind: "Clear", color });
  }

  drawRect(rect, color, cornerRadius = 0) {
    this._commands.push({ kind: "DrawRect", rect, color, cornerRadius });
  }

  drawText(text, fontFamily, fontSize, rect, color) {
    this._commands.push({ kind: "DrawText", text, fontFamily, fontSize, rect, color });
  }

  drawImage(resourceId, rect, opacity = 1.0) {
    this._commands.push({ kind: "DrawImage", resourceId, rect, opacity });
  }

  pushClip(rect) {
    this._commands.push({ kind: "PushClip", rect });
  }

  popClip() {
    this._commands.push({ kind: "PopClip" });
  }

  invalidate(rect) {
    this._invalidated.push(rect);
  }

  getCommands() {
    return this._commands;
  }

  toJsonBatch() {
    return JSON.stringify(this._commands);
  }
}

class InMemorySettingsStore {
  constructor() {
    this._map = new Map();
  }

  get(key) {
    return this._map.get(key);
  }

  set(key, value) {
    this._map.set(key, value);
  }
}

module.exports = {
  BatchRenderCanvas,
  InMemorySettingsStore,
};

