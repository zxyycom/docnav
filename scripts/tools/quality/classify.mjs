/**
 * 文件分类和 code area 归类。
 *
 * 根据配置文件中的 glob 规则将文件归类到 6 类默认 code areas。
 * 同时负责路径过滤和 generated files 识别。
 *
 * 来源：openspec/changes/implement-code-quality-observability/specs/code-quality-observability/spec.md
 */

import { minimatch } from "minimatch";

/**
 * 将文件路径归类到配置定义的 code area。
 *
 * 匹配优先级：
 * 1. generated files（最高优先）
 * 2. 按配置的 code areas 顺序匹配 globs
 * 3. 未匹配的文件归入 "unknown" code area
 *
 * @param {string} filePath - 仓库相对路径（使用 / 分隔符）
 * @param {import('./config.mjs').DEFAULT_CONFIG['codeAreas']} codeAreas - 6 类 code area 定义
 * @param {string[]} generatedFileGlobs - generated files glob 模式
 * @returns {string} code area name
 */
export function classifyFile(filePath, codeAreas, generatedFileGlobs) {
  // 1. 检查 generated files
  if (generatedFileGlobs.some((g) => minimatch(filePath, g))) {
    return "generated";
  }

  // 2. 按配置顺序匹配 code areas（优先匹配靠前的）
  for (const [name, def] of Object.entries(codeAreas)) {
    if (name === "generated") continue; // generated 只在步骤 1 匹配

    // exclude 优先：如果命中 exclude glob，此 area 不算
    if (def.excludeGlobs.some((g) => minimatch(filePath, g))) {
      continue;
    }
    // 命中 include glob
    if (def.globs.some((g) => minimatch(filePath, g))) {
      return name;
    }
  }

  // 3. 兜底
  return "unknown";
}

/**
 * 是否为被排除的路径（排除目录、generated file 等）。
 *
 * @param {string} filePath - 仓库相对路径
 * @param {string[]} excludeDirs - 排除目录名
 * @param {string[]} generatedFileGlobs - generated files glob
 * @returns {boolean}
 */
export function isExcluded(filePath, excludeDirs, generatedFileGlobs) {
  const parts = filePath.split("/");

  // 检查排除目录
  if (excludeDirs.some((d) => parts.includes(d))) {
    return true;
  }

  // 检查 generated files
  if (generatedFileGlobs.some((g) => minimatch(filePath, g))) {
    return true;
  }

  return false;
}

/**
 * 批量分类文件。
 *
 * @param {string[]} files - 仓库相对路径列表
 * @param {import('./config.mjs').DEFAULT_CONFIG['codeAreas']} codeAreas
 * @param {string[]} generatedFileGlobs
 * @returns {Map<string, string[]>} codeArea -> files 映射
 */
export function classifyFiles(files, codeAreas, generatedFileGlobs) {
  /** @type {Map<string, string[]>} */
  const groups = new Map();

  for (const file of files) {
    const area = classifyFile(file, codeAreas, generatedFileGlobs);
    if (!groups.has(area)) {
      groups.set(area, []);
    }
    groups.get(area).push(file);
  }

  // 按文件名排序
  for (const files of groups.values()) {
    files.sort();
  }

  return groups;
}

/**
 * 构建 code area 的扫描输入指纹。
 *
 * @param {string} codeArea - Code area name
 * @param {string[]} files - 该 code area 的文件列表
 * @param {Function} gitHashFn - (filePath: string) => string: git hash-object 的输出
 * @returns {import('./schema.mjs').CodeAreaFingerprint}
 */
export function buildFingerprint(codeArea, files, gitHashFn) {
  // 指纹：对所有文件路径和 blob hash 做复合哈希
  const parts = files.map((f) => `${f}:${gitHashFn(f)}`);
  const full = parts.join("\n");

  // 简单指纹：用 path+hash 字符串的长度和 checksum
  let hash = 0;
  for (let i = 0; i < full.length; i++) {
    const ch = full.charCodeAt(i);
    hash = ((hash << 5) - hash + ch) | 0;
  }

  return {
    fileCount: files.length,
    fileList: files.length <= 200 ? files.slice().sort() : files.slice(0, 200).sort(),
    fingerprint: `sha256-sim:${hash.toString(36)}:${files.length}`
  };
}
