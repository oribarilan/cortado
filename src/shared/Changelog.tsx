import { useState } from "react";

type ChangelogSection = {
  heading: string;
  entries: string[];
};

type ChangelogVersion = {
  version: string;
  date: string | null;
  sections: ChangelogSection[];
};

/** Color class for section headings based on change type. */
function sectionColorClass(heading: string): string {
  switch (heading.toLowerCase()) {
    case "added":
      return "cl-heading-added";
    case "changed":
      return "cl-heading-changed";
    case "fixed":
      return "cl-heading-fixed";
    default:
      return "";
  }
}

/**
 * Renders structured changelog data (from the update feed's `changelog` field).
 * Collapsible per-version, all expanded by default.
 */
export function Changelog({ json }: { json: string }) {
  let versions: ChangelogVersion[];
  try {
    versions = JSON.parse(json);
  } catch {
    return null;
  }

  if (!versions || versions.length === 0) return null;

  return (
    <div className="cl-root">
      <div className="cl-label">What's new</div>
      {versions.map((v) => (
        <VersionSection key={v.version} version={v} />
      ))}
    </div>
  );
}

function VersionSection({ version }: { version: ChangelogVersion }) {
  const [expanded, setExpanded] = useState(true);

  return (
    <div className={`cl-version ${expanded ? "expanded" : ""}`}>
      <button
        className="cl-version-header"
        onClick={() => setExpanded((e) => !e)}
        aria-expanded={expanded}
      >
        <span className="cl-chevron" aria-hidden="true">▸</span>
        <span className="cl-version-label">v{version.version}</span>
        {version.date ? (
          <span className="cl-version-date">{version.date}</span>
        ) : null}
      </button>
      <div className="cl-version-body">
        <div className="cl-version-inner">
          {version.sections.map((section) => (
            <div className="cl-section" key={section.heading}>
              <span className={`cl-section-heading ${sectionColorClass(section.heading)}`}>
                {section.heading}
              </span>
              <ul className="cl-entries">
                {section.entries.map((entry, i) => (
                  <li key={i} className="cl-entry">{entry}</li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
