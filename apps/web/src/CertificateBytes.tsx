import type {
  ByteRange,
  WireLayout,
} from "../../../packages/contracts/src/index";
import { describe, fieldValue, fields, hexBytes } from "./bytes";

export type ByteChange = { offset: number; before: string[]; after: string[] };
const ROW = 16;

/** Certificate bytes labelled by the decoder's reported layout. The table and
 * status line carry the information; the hex grid repeats it visually. */
export function CertificateBytes({
  rawHex,
  layout,
  highlight,
  changes,
  onSelect,
}: {
  rawHex: string;
  layout: WireLayout | null;
  highlight: ByteRange | null;
  changes: ByteChange[];
  onSelect: (range: ByteRange) => void;
}) {
  const bytes = hexBytes(rawHex);
  const labelled = layout ? fields(layout) : [];
  const fieldAt = (i: number) =>
    labelled.find((f) => f.range.start <= i && i < f.range.end)?.key;
  const lit = (i: number) =>
    highlight !== null && highlight.start <= i && i < highlight.end;
  const touches = (range: ByteRange) =>
    highlight !== null &&
    (highlight.start === highlight.end
      ? range.start <= highlight.start && highlight.start < range.end
      : highlight.start < range.end && range.start < highlight.end);
  const before = new Map<number, string>();
  for (const change of changes)
    change.before.forEach((b, j) => before.set(change.offset + j, b));
  const named = labelled.filter((f) => touches(f.range)).map((f) => f.label);
  const rows: number[] = [];
  for (let row = 0; row < bytes.length; row += ROW) rows.push(row);

  return (
    <div className="hex-view">
      {labelled.length > 0 ? (
        <table className="hex-fields">
          <caption>Wire fields reported by the decoder</caption>
          <thead>
            <tr>
              <th scope="col">Field</th>
              <th scope="col">Bytes</th>
              <th scope="col">Value</th>
            </tr>
          </thead>
          <tbody>
            {labelled.map((f) => (
              <tr
                key={f.key}
                data-field={f.key}
                data-lit={touches(f.range) || undefined}
              >
                <th scope="row">
                  <button
                    type="button"
                    className="field-button"
                    aria-pressed={
                      highlight?.start === f.range.start &&
                      highlight.end === f.range.end
                    }
                    onClick={() => onSelect(f.range)}
                  >
                    {f.label}
                  </button>
                </th>
                <td className="mono hex-range">
                  {f.range.end - f.range.start === 1
                    ? f.range.start
                    : `${f.range.start}–${f.range.end - 1}`}
                </td>
                <td className="mono">
                  {fieldValue(f.key, bytes.slice(f.range.start, f.range.end))}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <p className="small muted">
          The certificate did not decode, so its fields are not labelled. A
          located check still highlights its bytes.
        </p>
      )}
      <p className="small hex-focus" aria-live="polite">
        {highlight
          ? `Highlighted: ${describe(highlight)}${named.length ? `, ${named.join(", ")}` : ""}.`
          : "Select a field, or a check's “Show bytes”, to highlight its bytes."}
      </p>
      {changes.length > 0 && (
        <p className="small hex-changes">
          Changed by the experiment:{" "}
          {changes
            .map(
              (c) =>
                `${describe({ start: c.offset, end: c.offset + c.after.length })}, ${c.before.join("")} → ${c.after.join("")}`,
            )
            .join("; ")}
          .
        </p>
      )}
      <div className="hex-grid" aria-hidden="true">
        {rows.map((row) => (
          <div className="hex-row" key={row}>
            <span className="hex-offset">{String(row).padStart(3, "0")}</span>
            {bytes.slice(row, row + ROW).map((b, j) => {
              const i = row + j;
              return (
                <span
                  key={i}
                  className="hex-byte"
                  data-field={fieldAt(i)}
                  data-lit={lit(i) || undefined}
                  data-changed={before.has(i) || undefined}
                  title={before.has(i) ? `was ${before.get(i)}` : undefined}
                >
                  {b}
                </span>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
