const fs = require("fs");
const path = require("path");

const DATA_DIR = path.resolve(__dirname, "../../data");
const DATA_FILE = path.join(DATA_DIR, "investigations.json");

function ensureStorage() {
  if (!fs.existsSync(DATA_DIR)) {
    fs.mkdirSync(DATA_DIR, { recursive: true });
  }

  if (!fs.existsSync(DATA_FILE)) {
    fs.writeFileSync(DATA_FILE, "[]", "utf8");
  }
}

function readStore() {
  ensureStorage();

  try {
    const raw = fs.readFileSync(DATA_FILE, "utf8");
    const data = JSON.parse(raw);

    return Array.isArray(data) ? data : [];
  } catch (error) {
    console.error("[INVESTIGATIONS] Failed to read storage:", error.message);

    return [];
  }
}

function writeStore(store) {
  ensureStorage();

  fs.writeFileSync(DATA_FILE, JSON.stringify(store, null, 2), "utf8");
}

function normalizeInvestigation(data) {
  const id = data.investigation_id || data.id || `inv-${Date.now()}`;

  const createdAt = data.created_at || new Date().toISOString();

  return {
    investigation_id: id,
    id,
    m5_investigation_id:
      data.m5_investigation_id != null
        ? Number(data.m5_investigation_id)
        : null,
    task_id: data.task_id != null ? String(data.task_id) : null,
    title: data.title || "Untitled Investigation",

    target: data.target || "Windows Fleet",

    target_scope: data.target_scope || data.target || "Windows Fleet",

    status: data.status || "READY",

    last_compiled: data.last_compiled || new Date().toISOString(),

    lastCompiled:
      data.lastCompiled || data.last_compiled || new Date().toISOString(),

    grammar_version: data.grammar_version || "v0.1",

    grammarVersion:
      data.grammarVersion || data.grammar_version || "Grammar v0.1",

    description: data.description || "",

    created_by: data.created_by || "system",

    investigator: data.investigator || data.created_by || "system",

    created_at: createdAt,

    created:
      data.created ||
      new Date(createdAt).toLocaleDateString("en-GB", {
        day: "numeric",
        month: "short",
        year: "numeric",
      }),

    script: data.script || "",

    compiled_ir: data.compiled_ir || null,
  };
}

module.exports = {
  findAll: async () => {
    return readStore();
  },

  findById: async (id) => {
    if (!id) {
      return undefined;
    }

    const store = readStore();

    return store.find(
      (inv) =>
        String(inv.investigation_id).toLowerCase() ===
          String(id).toLowerCase() ||
        String(inv.id).toLowerCase() === String(id).toLowerCase(),
    );
  },

  create: async (data) => {
    const store = readStore();

    const record = normalizeInvestigation(data);

    store.unshift(record);

    writeStore(store);

    return record;
  },
  update: async (id, updates) => {
    const store = readStore();

    const index = store.findIndex(
      (inv) =>
        String(inv.investigation_id).toLowerCase() ===
          String(id).toLowerCase() ||
        String(inv.id).toLowerCase() === String(id).toLowerCase(),
    );

    if (index === -1) {
      return undefined;
    }

    store[index] = normalizeInvestigation({
      ...store[index],
      ...updates,
    });

    writeStore(store);

    return store[index];
  },
};
