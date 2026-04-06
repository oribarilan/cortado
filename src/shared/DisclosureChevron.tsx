/** Reusable disclosure chevron indicator for expandable rows.
 *
 * Renders a right-pointing SVG chevron that rotates 90 degrees
 * when the parent element has the `.expanded` class.
 *
 * CSS lives in `tokens.css` (`.disclosure-chevron`).
 */
export function DisclosureChevron() {
  return (
    <span
      className="disclosure-chevron"
      dangerouslySetInnerHTML={{
        __html:
          '<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 4 10 8 6 12"/></svg>',
      }}
    />
  );
}
