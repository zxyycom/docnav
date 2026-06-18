export function formatTable(rows: ExternalValue) {
  if (rows.length === 0) return "";

  const colCount = Math.max(...rows.map((row: ExternalValue) => row.length));
  const colWidths = new Array(colCount).fill(3);

  for (const row of rows) {
    for (let i = 0; i < colCount; i++) {
      colWidths[i] = Math.max(colWidths[i], (row[i] || "").length);
    }
  }

  const formatRow = (row: ExternalValue) => {
    const cells: ExternalValue[] = [];
    for (let i = 0; i < colCount; i++) {
      cells.push((row[i] || "").padEnd(colWidths[i]));
    }
    return `| ${cells.join(" | ")} |`;
  };

  const separator = `|-${colWidths.map((width) => "-".repeat(width)).join("-|-")}-|`;

  const result = [formatRow(rows[0]), separator];
  for (let i = 1; i < rows.length; i++) {
    result.push(formatRow(rows[i]));
  }

  return result.join("\n");
}
