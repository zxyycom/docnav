import { assert } from "../../assertions.ts";
import { EXAMPLES, FIELDS, MARKDOWN_MANIFEST_EXPECTED } from "../../config.ts";
import { readJson } from "../../json/files.ts";
import { isRecord } from "../../../type-guards.ts";
import { jsonArray, jsonObject } from "./json.ts";

export function validateManifestSemantics() {
  const manifest = jsonObject(readJson(EXAMPLES.manifest), EXAMPLES.manifest);
  const adapter = jsonObject(manifest[FIELDS.adapter], "manifest adapter");
  assert(
    adapter[FIELDS.id] ===
      MARKDOWN_MANIFEST_EXPECTED.adapterId,
    "manifest example must describe docnav-markdown",
  );

  const capabilities = jsonArray(manifest[FIELDS.capabilities], "manifest capabilities");
  for (const capability of MARKDOWN_MANIFEST_EXPECTED.capabilities) {
    assert(
      capabilities.includes(capability),
      `markdown manifest example missing capability ${capability}`,
    );
  }

  const formats = jsonArray(manifest[FIELDS.formats], "manifest formats");
  const markdownFormat = formats.find(
    (format): format is Record<string, unknown> =>
      isRecord(format) && format[FIELDS.id] === MARKDOWN_MANIFEST_EXPECTED.formatId,
  );
  assert(markdownFormat, "manifest example missing markdown format");
  const extensions = jsonArray(markdownFormat[FIELDS.extensions], "markdown format extensions");
  assert(
    extensions.includes(
      MARKDOWN_MANIFEST_EXPECTED.extension,
    ),
    "markdown manifest example missing .md extension",
  );
  const contentTypes = jsonArray(markdownFormat[FIELDS.contentTypes], "markdown format content_types");
  assert(
    contentTypes.includes(
      MARKDOWN_MANIFEST_EXPECTED.contentType,
    ),
    "markdown manifest example missing text/markdown content type",
  );

  console.log("manifest example consistency ok: markdown capabilities and format");
}
