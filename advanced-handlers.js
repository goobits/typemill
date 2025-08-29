// src/mcp/handlers/advanced-handlers.ts
import { resolve } from "node:path";

// src/file-editor.js
import { readFile, readdir, stat } from "node:fs/promises";
import { constants, access } from "node:fs/promises";
import { extname, join } from "node:path";
import {
  existsSync,
  lstatSync,
  readFileSync,
  realpathSync,
  renameSync,
  statSync,
  unlinkSync,
  writeFileSync
} from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
var __create = Object.create;
var __getProtoOf = Object.getPrototypeOf;
var __defProp = Object.defineProperty;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __toESM = (mod, isNodeMode, target) => {
  target = mod != null ? __create(__getProtoOf(mod)) : {};
  const to = isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target;
  for (let key of __getOwnPropNames(mod))
    if (!__hasOwnProp.call(to, key))
      __defProp(to, key, {
        get: () => mod[key],
        enumerable: true
      });
  return to;
};
var __commonJS = (cb, mod) => () => (mod || cb((mod = { exports: {} }).exports, mod), mod.exports);
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, {
      get: all[name],
      enumerable: true,
      configurable: true,
      set: (newValue) => all[name] = () => newValue
    });
};
var __esm = (fn, res) => () => (fn && (res = fn(fn = 0)), res);
var require_ignore = __commonJS((exports, module) => {
  function makeArray(subject) {
    return Array.isArray(subject) ? subject : [subject];
  }
  var UNDEFINED = undefined;
  var EMPTY = "";
  var SPACE = " ";
  var ESCAPE = "\\";
  var REGEX_TEST_BLANK_LINE = /^\s+$/;
  var REGEX_INVALID_TRAILING_BACKSLASH = /(?:[^\\]|^)\\$/;
  var REGEX_REPLACE_LEADING_EXCAPED_EXCLAMATION = /^\\!/;
  var REGEX_REPLACE_LEADING_EXCAPED_HASH = /^\\#/;
  var REGEX_SPLITALL_CRLF = /\r?\n/g;
  var REGEX_TEST_INVALID_PATH = /^\.{0,2}\/|^\.{1,2}$/;
  var REGEX_TEST_TRAILING_SLASH = /\/$/;
  var SLASH = "/";
  var TMP_KEY_IGNORE = "node-ignore";
  if (typeof Symbol !== "undefined") {
    TMP_KEY_IGNORE = Symbol.for("node-ignore");
  }
  var KEY_IGNORE = TMP_KEY_IGNORE;
  var define = (object, key, value) => {
    Object.defineProperty(object, key, { value });
    return value;
  };
  var REGEX_REGEXP_RANGE = /([0-z])-([0-z])/g;
  var RETURN_FALSE = () => false;
  var sanitizeRange = (range) => range.replace(REGEX_REGEXP_RANGE, (match, from, to) => from.charCodeAt(0) <= to.charCodeAt(0) ? match : EMPTY);
  var cleanRangeBackSlash = (slashes) => {
    const { length } = slashes;
    return slashes.slice(0, length - length % 2);
  };
  var REPLACERS = [
    [
      /^\uFEFF/,
      () => EMPTY
    ],
    [
      /((?:\\\\)*?)(\\?\s+)$/,
      (_, m1, m2) => m1 + (m2.indexOf("\\") === 0 ? SPACE : EMPTY)
    ],
    [
      /(\\+?)\s/g,
      (_, m1) => {
        const { length } = m1;
        return m1.slice(0, length - length % 2) + SPACE;
      }
    ],
    [
      /[\\$.|*+(){^]/g,
      (match) => `\\${match}`
    ],
    [
      /(?!\\)\?/g,
      () => "[^/]"
    ],
    [
      /^\//,
      () => "^"
    ],
    [
      /\//g,
      () => "\\/"
    ],
    [
      /^\^*\\\*\\\*\\\//,
      () => "^(?:.*\\/)?"
    ],
    [
      /^(?=[^^])/,
      function startingReplacer() {
        return !/\/(?!$)/.test(this) ? "(?:^|\\/)" : "^";
      }
    ],
    [
      /\\\/\\\*\\\*(?=\\\/|$)/g,
      (_, index, str) => index + 6 < str.length ? "(?:\\/[^\\/]+)*" : "\\/.+"
    ],
    [
      /(^|[^\\]+)(\\\*)+(?=.+)/g,
      (_, p1, p2) => {
        const unescaped = p2.replace(/\\\*/g, "[^\\/]*");
        return p1 + unescaped;
      }
    ],
    [
      /\\\\\\(?=[$.|*+(){^])/g,
      () => ESCAPE
    ],
    [
      /\\\\/g,
      () => ESCAPE
    ],
    [
      /(\\)?\[([^\]/]*?)(\\*)($|\])/g,
      (match, leadEscape, range, endEscape, close) => leadEscape === ESCAPE ? `\\[${range}${cleanRangeBackSlash(endEscape)}${close}` : close === "]" ? endEscape.length % 2 === 0 ? `[${sanitizeRange(range)}${endEscape}]` : "[]" : "[]"
    ],
    [
      /(?:[^*])$/,
      (match) => /\/$/.test(match) ? `${match}$` : `${match}(?=$|\\/$)`
    ]
  ];
  var REGEX_REPLACE_TRAILING_WILDCARD = /(^|\\\/)?\\\*$/;
  var MODE_IGNORE = "regex";
  var MODE_CHECK_IGNORE = "checkRegex";
  var UNDERSCORE = "_";
  var TRAILING_WILD_CARD_REPLACERS = {
    [MODE_IGNORE](_, p1) {
      const prefix = p1 ? `${p1}[^/]+` : "[^/]*";
      return `${prefix}(?=$|\\/$)`;
    },
    [MODE_CHECK_IGNORE](_, p1) {
      const prefix = p1 ? `${p1}[^/]*` : "[^/]*";
      return `${prefix}(?=$|\\/$)`;
    }
  };
  var makeRegexPrefix = (pattern) => REPLACERS.reduce((prev, [matcher, replacer]) => prev.replace(matcher, replacer.bind(pattern)), pattern);
  var isString = (subject) => typeof subject === "string";
  var checkPattern = (pattern) => pattern && isString(pattern) && !REGEX_TEST_BLANK_LINE.test(pattern) && !REGEX_INVALID_TRAILING_BACKSLASH.test(pattern) && pattern.indexOf("#") !== 0;
  var splitPattern = (pattern) => pattern.split(REGEX_SPLITALL_CRLF).filter(Boolean);

  class IgnoreRule {
    constructor(pattern, mark, body, ignoreCase, negative, prefix) {
      this.pattern = pattern;
      this.mark = mark;
      this.negative = negative;
      define(this, "body", body);
      define(this, "ignoreCase", ignoreCase);
      define(this, "regexPrefix", prefix);
    }
    get regex() {
      const key = UNDERSCORE + MODE_IGNORE;
      if (this[key]) {
        return this[key];
      }
      return this._make(MODE_IGNORE, key);
    }
    get checkRegex() {
      const key = UNDERSCORE + MODE_CHECK_IGNORE;
      if (this[key]) {
        return this[key];
      }
      return this._make(MODE_CHECK_IGNORE, key);
    }
    _make(mode, key) {
      const str = this.regexPrefix.replace(REGEX_REPLACE_TRAILING_WILDCARD, TRAILING_WILD_CARD_REPLACERS[mode]);
      const regex = this.ignoreCase ? new RegExp(str, "i") : new RegExp(str);
      return define(this, key, regex);
    }
  }
  var createRule = ({
    pattern,
    mark
  }, ignoreCase) => {
    let negative = false;
    let body = pattern;
    if (body.indexOf("!") === 0) {
      negative = true;
      body = body.substr(1);
    }
    body = body.replace(REGEX_REPLACE_LEADING_EXCAPED_EXCLAMATION, "!").replace(REGEX_REPLACE_LEADING_EXCAPED_HASH, "#");
    const regexPrefix = makeRegexPrefix(body);
    return new IgnoreRule(pattern, mark, body, ignoreCase, negative, regexPrefix);
  };

  class RuleManager {
    constructor(ignoreCase) {
      this._ignoreCase = ignoreCase;
      this._rules = [];
    }
    _add(pattern) {
      if (pattern && pattern[KEY_IGNORE]) {
        this._rules = this._rules.concat(pattern._rules._rules);
        this._added = true;
        return;
      }
      if (isString(pattern)) {
        pattern = {
          pattern
        };
      }
      if (checkPattern(pattern.pattern)) {
        const rule = createRule(pattern, this._ignoreCase);
        this._added = true;
        this._rules.push(rule);
      }
    }
    add(pattern) {
      this._added = false;
      makeArray(isString(pattern) ? splitPattern(pattern) : pattern).forEach(this._add, this);
      return this._added;
    }
    test(path, checkUnignored, mode) {
      let ignored = false;
      let unignored = false;
      let matchedRule;
      this._rules.forEach((rule) => {
        const { negative } = rule;
        if (unignored === negative && ignored !== unignored || negative && !ignored && !unignored && !checkUnignored) {
          return;
        }
        const matched = rule[mode].test(path);
        if (!matched) {
          return;
        }
        ignored = !negative;
        unignored = negative;
        matchedRule = negative ? UNDEFINED : rule;
      });
      const ret = {
        ignored,
        unignored
      };
      if (matchedRule) {
        ret.rule = matchedRule;
      }
      return ret;
    }
  }
  var throwError = (message, Ctor) => {
    throw new Ctor(message);
  };
  var checkPath = (path, originalPath, doThrow) => {
    if (!isString(path)) {
      return doThrow(`path must be a string, but got \`${originalPath}\``, TypeError);
    }
    if (!path) {
      return doThrow(`path must not be empty`, TypeError);
    }
    if (checkPath.isNotRelative(path)) {
      const r = "`path.relative()`d";
      return doThrow(`path should be a ${r} string, but got "${originalPath}"`, RangeError);
    }
    return true;
  };
  var isNotRelative = (path) => REGEX_TEST_INVALID_PATH.test(path);
  checkPath.isNotRelative = isNotRelative;
  checkPath.convert = (p) => p;

  class Ignore {
    constructor({
      ignorecase = true,
      ignoreCase = ignorecase,
      allowRelativePaths = false
    } = {}) {
      define(this, KEY_IGNORE, true);
      this._rules = new RuleManager(ignoreCase);
      this._strictPathCheck = !allowRelativePaths;
      this._initCache();
    }
    _initCache() {
      this._ignoreCache = Object.create(null);
      this._testCache = Object.create(null);
    }
    add(pattern) {
      if (this._rules.add(pattern)) {
        this._initCache();
      }
      return this;
    }
    addPattern(pattern) {
      return this.add(pattern);
    }
    _test(originalPath, cache, checkUnignored, slices) {
      const path = originalPath && checkPath.convert(originalPath);
      checkPath(path, originalPath, this._strictPathCheck ? throwError : RETURN_FALSE);
      return this._t(path, cache, checkUnignored, slices);
    }
    checkIgnore(path) {
      if (!REGEX_TEST_TRAILING_SLASH.test(path)) {
        return this.test(path);
      }
      const slices = path.split(SLASH).filter(Boolean);
      slices.pop();
      if (slices.length) {
        const parent = this._t(slices.join(SLASH) + SLASH, this._testCache, true, slices);
        if (parent.ignored) {
          return parent;
        }
      }
      return this._rules.test(path, false, MODE_CHECK_IGNORE);
    }
    _t(path, cache, checkUnignored, slices) {
      if (path in cache) {
        return cache[path];
      }
      if (!slices) {
        slices = path.split(SLASH).filter(Boolean);
      }
      slices.pop();
      if (!slices.length) {
        return cache[path] = this._rules.test(path, checkUnignored, MODE_IGNORE);
      }
      const parent = this._t(slices.join(SLASH) + SLASH, cache, checkUnignored, slices);
      return cache[path] = parent.ignored ? parent : this._rules.test(path, checkUnignored, MODE_IGNORE);
    }
    ignores(path) {
      return this._test(path, this._ignoreCache, false).ignored;
    }
    createFilter() {
      return (path) => !this.ignores(path);
    }
    filter(paths) {
      return makeArray(paths).filter(this.createFilter());
    }
    test(path) {
      return this._test(path, this._testCache, true);
    }
  }
  var factory = (options) => new Ignore(options);
  var isPathValid = (path) => checkPath(path && checkPath.convert(path), path, RETURN_FALSE);
  var setupWindows = () => {
    const makePosix = (str) => /^\\\\\?\\/.test(str) || /["<>|\u0000-\u001F]+/u.test(str) ? str : str.replace(/\\/g, "/");
    checkPath.convert = makePosix;
    const REGEX_TEST_WINDOWS_PATH_ABSOLUTE = /^[a-z]:\//i;
    checkPath.isNotRelative = (path) => REGEX_TEST_WINDOWS_PATH_ABSOLUTE.test(path) || isNotRelative(path);
  };
  if (typeof process !== "undefined" && process.platform === "win32") {
    setupWindows();
  }
  module.exports = factory;
  factory.default = factory;
  module.exports.isPathValid = isPathValid;
  define(module.exports, Symbol.for("setupWindows"), setupWindows);
});
var exports_file_scanner = {};
__export(exports_file_scanner, {
  scanProjectFiles: () => scanProjectFiles,
  scanDirectoryForExtensions: () => scanDirectoryForExtensions,
  loadGitignore: () => loadGitignore,
  getRecommendedLanguageServers: () => getRecommendedLanguageServers
});
async function loadGitignore(projectPath) {
  const ig = import_ignore.default();
  ig.add(DEFAULT_IGNORE_PATTERNS);
  const gitignorePath = join(projectPath, ".gitignore");
  try {
    await access(gitignorePath, constants.F_OK);
    const gitignoreContent = await readFile(gitignorePath, "utf-8");
    ig.add(gitignoreContent);
  } catch (error) {}
  return ig;
}
async function scanDirectoryForExtensions(dirPath, maxDepth = 3, ignoreFilter, debug = false) {
  const extensions = new Set;
  async function scanDirectory(currentPath, currentDepth, relativePath = "") {
    if (currentDepth > maxDepth)
      return;
    try {
      const entries = await readdir(currentPath);
      if (debug) {
        process.stderr.write(`Scanning directory ${currentPath} (depth: ${currentDepth}), found ${entries.length} entries: ${entries.join(", ")}
`);
      }
      for (const entry of entries) {
        const fullPath = join(currentPath, entry);
        const entryRelativePath = relativePath ? join(relativePath, entry) : entry;
        const normalizedPath = entryRelativePath.replace(/\\/g, "/");
        if (ignoreFilter?.ignores(normalizedPath)) {
          if (debug) {
            process.stderr.write(`Skipping ignored entry: ${entryRelativePath}
`);
          }
          continue;
        }
        try {
          const fileStat = await stat(fullPath);
          if (fileStat.isDirectory()) {
            if (debug) {
              process.stderr.write(`Recursing into directory: ${entryRelativePath}
`);
            }
            await scanDirectory(fullPath, currentDepth + 1, entryRelativePath);
          } else if (fileStat.isFile()) {
            const ext = extname(entry).toLowerCase().slice(1);
            if (debug) {
              process.stderr.write(`Found file: ${entry}, extension: "${ext}"
`);
            }
            if (ext) {
              extensions.add(ext);
              if (debug) {
                process.stderr.write(`Added extension: ${ext}
`);
              }
            }
          }
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : String(error);
          process.stderr.write(`Error processing file ${fullPath} (stat/type check): ${errorMsg}
`);
        }
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      process.stderr.write(`Error reading directory ${currentPath} (readdir operation): ${errorMsg}
`);
      return;
    }
  }
  await scanDirectory(dirPath, 0);
  return extensions;
}
function getRecommendedLanguageServers(extensions, languageServers) {
  const recommended = [];
  for (const server of languageServers) {
    const hasMatchingExtension = server.extensions.some((ext) => extensions.has(ext));
    if (hasMatchingExtension) {
      recommended.push(server.name);
    }
  }
  return recommended;
}
async function scanProjectFiles(projectPath, languageServers, maxDepth = 3, debug = false) {
  const ignoreFilter = await loadGitignore(projectPath);
  const extensions = await scanDirectoryForExtensions(projectPath, maxDepth, ignoreFilter, debug);
  const recommendedServers = getRecommendedLanguageServers(extensions, languageServers);
  return {
    extensions,
    recommendedServers
  };
}
var import_ignore;
var DEFAULT_IGNORE_PATTERNS;
var init_file_scanner = __esm(() => {
  import_ignore = __toESM(require_ignore(), 1);
  DEFAULT_IGNORE_PATTERNS = [
    "node_modules",
    ".git",
    ".svn",
    ".hg",
    "dist",
    "build",
    "out",
    "target",
    "bin",
    "obj",
    ".next",
    ".nuxt",
    "coverage",
    ".nyc_output",
    "temp",
    "cache",
    ".cache",
    ".vscode",
    ".idea",
    "*.log",
    ".DS_Store",
    "Thumbs.db"
  ];
});
function uriToPath(uri) {
  return fileURLToPath(uri);
}
async function applyWorkspaceEdit(workspaceEdit, options = {}) {
  const {
    validateBeforeApply = true,
    createBackupFiles = validateBeforeApply,
    lspClient: lspClient2
  } = options;
  const backups = [];
  const filesModified = [];
  const backupFilePaths = [];
  if (!workspaceEdit.changes || Object.keys(workspaceEdit.changes).length === 0) {
    return {
      success: true,
      filesModified: [],
      backupFiles: []
    };
  }
  try {
    for (const [uri, edits] of Object.entries(workspaceEdit.changes)) {
      const filePath = uriToPath(uri);
      if (!existsSync(filePath)) {
        throw new Error(`File does not exist: ${filePath}`);
      }
      const stats = lstatSync(filePath);
      if (stats.isSymbolicLink()) {
        try {
          const realPath = realpathSync(filePath);
          const targetStats = statSync(realPath);
          if (!targetStats.isFile()) {
            throw new Error(`Symlink target is not a file: ${realPath}`);
          }
        } catch (error) {
          throw new Error(`Cannot resolve symlink ${filePath}: ${error}`);
        }
      } else if (!stats.isFile()) {
        throw new Error(`Not a file: ${filePath}`);
      }
      try {
        readFileSync(filePath, "utf-8");
      } catch (error) {
        throw new Error(`Cannot read file: ${filePath} - ${error}`);
      }
    }
    for (const [uri, edits] of Object.entries(workspaceEdit.changes)) {
      const originalPath = uriToPath(uri);
      let targetPath = originalPath;
      const originalStats = lstatSync(originalPath);
      if (originalStats.isSymbolicLink()) {
        targetPath = realpathSync(originalPath);
        process.stderr.write(`[DEBUG] Editing symlink target: ${targetPath} (via ${originalPath})
`);
      }
      const originalContent = readFileSync(targetPath, "utf-8");
      const backup = {
        originalPath,
        targetPath,
        originalContent
      };
      backups.push(backup);
      if (createBackupFiles) {
        const backupPath = `${originalPath}.bak`;
        writeFileSync(backupPath, originalContent, "utf-8");
        backupFilePaths.push(backupPath);
      }
      const modifiedContent = applyEditsToContent(originalContent, edits, validateBeforeApply);
      const tempPath = `${targetPath}.tmp-${process.pid}-${Date.now()}-${Math.random().toString(36).slice(2)}`;
      writeFileSync(tempPath, modifiedContent, "utf-8");
      try {
        renameSync(tempPath, targetPath);
      } catch (error) {
        try {
          if (existsSync(tempPath)) {
            unlinkSync(tempPath);
          }
        } catch {}
        throw error;
      }
      filesModified.push(originalPath);
      if (lspClient2) {
        await lspClient2.syncFileContent(originalPath);
      }
    }
    return {
      success: true,
      filesModified,
      backupFiles: backupFilePaths
    };
  } catch (error) {
    for (const backup of backups) {
      try {
        writeFileSync(backup.targetPath, backup.originalContent, "utf-8");
      } catch (rollbackError) {
        console.error(`Failed to rollback ${backup.targetPath}:`, rollbackError);
      }
    }
    return {
      success: false,
      filesModified: [],
      backupFiles: backupFilePaths,
      error: error instanceof Error ? error.message : String(error)
    };
  }
}
function applyEditsToContent(content, edits, validate) {
  const lineEnding = content.includes(`\r
`) ? `\r
` : `
`;
  const lines = content.split(/\r?\n/);
  const sortedEdits = [...edits].sort((a, b) => {
    if (a.range.start.line !== b.range.start.line) {
      return b.range.start.line - a.range.start.line;
    }
    return b.range.start.character - a.range.start.character;
  });
  for (const edit of sortedEdits) {
    const { start, end } = edit.range;
    if (validate) {
      if (start.line < 0 || start.line >= lines.length) {
        throw new Error(`Invalid start line ${start.line} (file has ${lines.length} lines)`);
      }
      if (end.line < 0 || end.line >= lines.length) {
        throw new Error(`Invalid end line ${end.line} (file has ${lines.length} lines)`);
      }
      if (start.line > end.line || start.line === end.line && start.character > end.character) {
        throw new Error(`Invalid range: start (${start.line}:${start.character}) is after end (${end.line}:${end.character})`);
      }
      const startLine = lines[start.line];
      if (startLine !== undefined) {
        if (start.character < 0 || start.character > startLine.length) {
          throw new Error(`Invalid start character ${start.character} on line ${start.line} (line has ${startLine.length} characters)`);
        }
      }
      const endLine = lines[end.line];
      if (endLine !== undefined) {
        if (end.character < 0 || end.character > endLine.length) {
          throw new Error(`Invalid end character ${end.character} on line ${end.line} (line has ${endLine.length} characters)`);
        }
      }
    }
    if (start.line === end.line) {
      const line = lines[start.line];
      if (line !== undefined) {
        lines[start.line] = line.substring(0, start.character) + edit.newText + line.substring(end.character);
      }
    } else {
      const startLine = lines[start.line];
      const endLine = lines[end.line];
      if (startLine !== undefined && endLine !== undefined) {
        const newLine = startLine.substring(0, start.character) + edit.newText + endLine.substring(end.character);
        lines.splice(start.line, end.line - start.line + 1, newLine);
      }
    }
  }
  return lines.join(lineEnding);
}

// src/path-utils.ts
import { fileURLToPath as fileURLToPath2, pathToFileURL as pathToFileURL2 } from "node:url";
function pathToUri(filePath) {
  return pathToFileURL2(filePath).toString();
}
function uriToPath2(uri) {
  return fileURLToPath2(uri);
}

// src/mcp/utils.ts
function createMCPResponse(text) {
  return {
    content: [
      {
        type: "text",
        text
      }
    ]
  };
}
function createLimitedSupportResponse(featureName, serverDescription, warningMessage, result) {
  let text = `⚠️ **${featureName}** - Limited support on ${serverDescription}

`;
  text += `**Warning:** ${warningMessage}

`;
  if (result) {
    text += `**Result:**
${result}`;
  } else {
    text += "**Result:** Feature attempted but may not work as expected on this server.";
  }
  return createMCPResponse(text);
}

// src/mcp/handlers/advanced-handlers.ts
async function handleGetCodeActions(fileService, args) {
  const { file_path, range } = args;
  const absolutePath = resolve(file_path);
  try {
    const codeActions = await fileService.getCodeActions(absolutePath, range);
    if (codeActions.length === 0) {
      return {
        content: [
          {
            type: "text",
            text: `No code actions available for ${file_path}${range ? ` at lines ${range.start.line + 1}-${range.end.line + 1}` : ""}.`
          }
        ]
      };
    }
    const actionDescriptions = codeActions.filter((action) => action && (action.title || action.kind)).map((action, index) => {
      if (action.title) {
        return `${index + 1}. ${action.title}${action.kind ? ` (${action.kind})` : ""}`;
      }
      return `${index + 1}. Code action (${action.kind || "unknown"})`;
    });
    return {
      content: [
        {
          type: "text",
          text: `Found ${codeActions.length} code action${codeActions.length === 1 ? "" : "s"} for ${file_path}:

${actionDescriptions.join(`
`)}

Note: These actions show what's available but cannot be applied directly through this tool. Use your editor's code action functionality to apply them.`
        }
      ]
    };
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: `Error getting code actions: ${error instanceof Error ? error.message : String(error)}`
        }
      ]
    };
  }
}
async function handleFormatDocument(fileService, args) {
  const { file_path, options } = args;
  const absolutePath = resolve(file_path);
  try {
    const lspOptions = options ? {
      tabSize: options.tab_size,
      insertSpaces: options.insert_spaces,
      trimTrailingWhitespace: options.trim_trailing_whitespace,
      insertFinalNewline: options.insert_final_newline,
      trimFinalNewlines: options.trim_final_newlines
    } : undefined;
    const formatEdits = await fileService.formatDocument(absolutePath, lspOptions);
    if (formatEdits.length === 0) {
      return {
        content: [
          {
            type: "text",
            text: `No formatting changes needed for ${file_path}. The file is already properly formatted.`
          }
        ]
      };
    }
    const workspaceEdit = {
      changes: {
        [pathToUri(absolutePath)]: formatEdits
      }
    };
    const editResult = await applyWorkspaceEdit(workspaceEdit);
    if (!editResult.success) {
      return {
        content: [
          {
            type: "text",
            text: `Failed to apply formatting: ${editResult.error}`
          }
        ]
      };
    }
    return {
      content: [
        {
          type: "text",
          text: `✅ Successfully formatted ${file_path} with ${formatEdits.length} change${formatEdits.length === 1 ? "" : "s"}.`
        }
      ]
    };
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: `Error formatting document: ${error instanceof Error ? error.message : String(error)}`
        }
      ]
    };
  }
}
async function handleSearchWorkspaceSymbols(symbolService, args) {
  const { query } = args;
  try {
    const symbols = await symbolService.searchWorkspaceSymbols(query);
    if (symbols.length === 0) {
      return {
        content: [
          {
            type: "text",
            text: `No symbols found matching "${query}". Try a different search term or ensure the language server is properly configured.`
          }
        ]
      };
    }
    const symbolDescriptions = symbols.slice(0, 50).map((symbol, index) => {
      const location = symbol.location;
      const filePath = uriToPath2(location.uri);
      const line = location.range.start.line + 1;
      const character = location.range.start.character + 1;
      const symbolKind = symbol.kind ? lspClient.symbolKindToString(symbol.kind) : "unknown";
      return `${index + 1}. ${symbol.name} (${symbolKind}) - ${filePath}:${line}:${character}`;
    });
    const resultText = symbols.length > 50 ? `Found ${symbols.length} symbols matching "${query}" (showing first 50):

${symbolDescriptions.join(`
`)}` : `Found ${symbols.length} symbol${symbols.length === 1 ? "" : "s"} matching "${query}":

${symbolDescriptions.join(`
`)}`;
    return {
      content: [
        {
          type: "text",
          text: resultText
        }
      ]
    };
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: `Error searching workspace symbols: ${error instanceof Error ? error.message : String(error)}`
        }
      ]
    };
  }
}
async function handleGetDocumentSymbols(symbolService, args) {
  const { file_path } = args;
  const absolutePath = resolve(file_path);
  try {
    const symbols = await symbolService.getDocumentSymbols(absolutePath);
    if (symbols.length === 0) {
      return {
        content: [
          {
            type: "text",
            text: `No symbols found in ${file_path}. The file may be empty or the language server may not support this file type.`
          }
        ]
      };
    }
    const isHierarchical = symbolService.isDocumentSymbolArray(symbols);
    let symbolDescriptions;
    if (isHierarchical) {
      const formatDocumentSymbol = (symbol, indent = 0) => {
        const prefix = "  ".repeat(indent);
        const line = symbol.range.start.line + 1;
        const character = symbol.range.start.character + 1;
        const symbolKind = symbolService.symbolKindToString(symbol.kind);
        const result = [`${prefix}${symbol.name} (${symbolKind}) - Line ${line}:${character}`];
        if (symbol.children && symbol.children.length > 0) {
          for (const child of symbol.children) {
            result.push(...formatDocumentSymbol(child, indent + 1));
          }
        }
        return result;
      };
      symbolDescriptions = [];
      for (const symbol of symbols) {
        symbolDescriptions.push(...formatDocumentSymbol(symbol));
      }
    } else {
      symbolDescriptions = symbols.map((symbol, index) => {
        const line = symbol.location.range.start.line + 1;
        const character = symbol.location.range.start.character + 1;
        const symbolKind = symbol.kind ? symbolService.symbolKindToString(symbol.kind) : "unknown";
        return `${index + 1}. ${symbol.name} (${symbolKind}) - Line ${line}:${character}`;
      });
    }
    return {
      content: [
        {
          type: "text",
          text: `Document outline for ${file_path}:

${symbolDescriptions.join(`
`)}`
        }
      ]
    };
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: `Error getting document symbols: ${error instanceof Error ? error.message : String(error)}`
        }
      ]
    };
  }
}
async function handleGetFoldingRanges(fileService, args) {
  const { file_path } = args;
  const absolutePath = resolve(file_path);
  try {
    const foldingRanges = await fileService.getFoldingRanges(absolutePath);
    if (foldingRanges.length === 0) {
      return createMCPResponse(`No folding ranges found in ${file_path}. The file may not have collapsible code blocks.`);
    }
    const rangeDescriptions = foldingRanges.map((range, index) => {
      const startLine = range.startLine + 1;
      const endLine = range.endLine + 1;
      const kind = range.kind || "code";
      const characterInfo = range.startCharacter !== undefined && range.endCharacter !== undefined ? ` (chars ${range.startCharacter}-${range.endCharacter})` : "";
      return `${index + 1}. **${kind}** block: Lines ${startLine}-${endLine}${characterInfo}${range.collapsedText ? ` ("${range.collapsedText}")` : ""}`;
    });
    const kindCount = foldingRanges.reduce((acc, range) => {
      const kind = range.kind || "code";
      acc[kind] = (acc[kind] || 0) + 1;
      return acc;
    }, {});
    const kindSummary = Object.entries(kindCount).map(([kind, count]) => `${count} ${kind}`).join(", ");
    const response = `## Folding Ranges for ${file_path}

**Found ${foldingRanges.length} foldable regions:** ${kindSummary}

${rangeDescriptions.join(`
`)}

*Folding ranges show logical code blocks that can be collapsed for better code navigation and understanding.*`;
    return createMCPResponse(response);
  } catch (error) {
    if (error instanceof Error && error.message.includes("not supported")) {
      const serverInfo = await lspClient.getCapabilityInfo(absolutePath);
      return createLimitedSupportResponse("Folding Ranges", "Current Language Server", "Server may not fully support folding ranges or the file has no collapsible regions", `Server capabilities: ${serverInfo}`);
    }
    return createMCPResponse(`Error getting folding ranges: ${error instanceof Error ? error.message : String(error)}`);
  }
}
async function handleGetDocumentLinks(fileService, args) {
  const { file_path } = args;
  const absolutePath = resolve(file_path);
  try {
    const documentLinks = await fileService.getDocumentLinks(absolutePath);
    if (documentLinks.length === 0) {
      return createMCPResponse(`No document links found in ${file_path}. The file may not contain URLs, imports, or other linkable references.`);
    }
    const linkDescriptions = documentLinks.map((link, index) => {
      const startLine = link.range.start.line + 1;
      const startChar = link.range.start.character + 1;
      const endLine = link.range.end.line + 1;
      const endChar = link.range.end.character + 1;
      let description = `${index + 1}. **Link** at Line ${startLine}:${startChar}`;
      if (startLine !== endLine || startChar !== endChar) {
        description += ` to ${endLine}:${endChar}`;
      }
      if (link.target) {
        description += `
   Target: ${link.target}`;
      }
      if (link.tooltip) {
        description += `
   Info: ${link.tooltip}`;
      }
      return description;
    });
    const categories = {
      urls: documentLinks.filter((link) => link.target?.startsWith("http")),
      files: documentLinks.filter((link) => link.target?.startsWith("file:")),
      packages: documentLinks.filter((link) => link.target?.includes("pkg.go.dev") || link.target?.includes("docs.rs") || link.target?.includes("npmjs.com")),
      other: documentLinks.filter((link) => link.target && !link.target.startsWith("http") && !link.target.startsWith("file:"))
    };
    let categorySummary = "";
    if (categories.urls.length > 0)
      categorySummary += `${categories.urls.length} URLs, `;
    if (categories.files.length > 0)
      categorySummary += `${categories.files.length} files, `;
    if (categories.packages.length > 0)
      categorySummary += `${categories.packages.length} packages, `;
    if (categories.other.length > 0)
      categorySummary += `${categories.other.length} other links, `;
    categorySummary = categorySummary.replace(/, $/, "");
    const response = `## Document Links for ${file_path}

**Found ${documentLinks.length} links:** ${categorySummary}

${linkDescriptions.join(`

`)}

*Document links help navigate between related files, external documentation, and web resources. Different language servers provide different types of links.*`;
    return createMCPResponse(response);
  } catch (error) {
    if (error instanceof Error && error.message.includes("not supported")) {
      const serverInfo = await lspClient.getCapabilityInfo(absolutePath);
      return createLimitedSupportResponse("Document Links", "Current Language Server", "Server may not fully support document links or the file contains no linkable content", `Server capabilities: ${serverInfo}`);
    }
    return createMCPResponse(`Error getting document links: ${error instanceof Error ? error.message : String(error)}`);
  }
}
async function handleApplyWorkspaceEdit(fileService, args) {
  const { changes, validate_before_apply = true } = args;
  try {
    const workspaceEdit = {
      changes: {}
    };
    for (const [filePath, edits] of Object.entries(changes)) {
      const uri = filePath.startsWith("file://") ? filePath : pathToUri(resolve(filePath));
      const textEdits = edits.map((edit) => ({
        range: edit.range,
        newText: edit.newText
      }));
      if (!workspaceEdit.changes) {
        workspaceEdit.changes = {};
      }
      workspaceEdit.changes[uri] = textEdits;
    }
    if (!workspaceEdit.changes || Object.keys(workspaceEdit.changes).length === 0) {
      return createMCPResponse("No changes provided. Please specify at least one file with edits to apply.");
    }
    const fileCount = Object.keys(workspaceEdit.changes).length;
    const editCount = Object.values(workspaceEdit.changes).reduce((sum, edits) => sum + edits.length, 0);
    const serverSupportsWorkspaceEdit = true;
    const serverDescription = "File-based workspace edit";
    const result = await fileService.applyWorkspaceEdit({
      changes: workspaceEdit.changes
    });
    if (!result.applied) {
      return createMCPResponse(`❌ **Workspace edit failed**

**Error:** ${result.failureReason || "Unknown error"}

**Files targeted:** ${fileCount}
**Total edits:** ${editCount}

*No changes were applied due to the error. All files remain unchanged.*`);
    }
    let response = `✅ **Workspace edit applied successfully**

`;
    const modifiedFiles = Object.keys(workspaceEdit.changes);
    response += `**Files modified:** ${modifiedFiles.length}
`;
    response += `**Total edits applied:** ${editCount}

`;
    if (modifiedFiles.length > 0) {
      response += `**Modified files:**
`;
      for (const file of modifiedFiles) {
        const filePath = file.startsWith("file://") ? uriToPath2(file) : file;
        response += `• ${filePath}
`;
      }
    }
    if (!serverSupportsWorkspaceEdit) {
      response += `
⚠️ **Note:** ${serverDescription} doesn't fully support workspace edits, but changes were applied successfully using CCLSP's built-in editor.`;
    }
    response += `

*All changes were applied atomically. If any edit had failed, all changes would have been rolled back.*`;
    return createMCPResponse(response);
  } catch (error) {
    return createMCPResponse(`Error applying workspace edit: ${error instanceof Error ? error.message : String(error)}`);
  }
}
export {
  handleSearchWorkspaceSymbols,
  handleGetFoldingRanges,
  handleGetDocumentSymbols,
  handleGetDocumentLinks,
  handleGetCodeActions,
  handleFormatDocument,
  handleApplyWorkspaceEdit
};
