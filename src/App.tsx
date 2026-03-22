import { useEffect, useState } from "react";

import { invoke } from "@tauri-apps/api/core";

type StatusKind = "success" | "warning" | "error" | "pending" | "neutral";

type FieldValue =
  | {
      type: "text";
      value: string;
    }
  | {
      type: "status";
      value: string;
      severity: StatusKind;
    }
  | {
      type: "number";
      value: number;
    }
  | {
      type: "url";
      value: string;
    };

type Field = {
  name: string;
  label: string;
  value: FieldValue;
};

type Activity = {
  id: string;
  title: string;
  fields: Field[];
};

type FieldDefinition = {
  name: string;
  label: string;
  field_type: "text" | "status" | "number" | "url";
  description: string;
};

type FeedSnapshot = {
  name: string;
  feed_type: string;
  activities: Activity[];
  provided_fields: FieldDefinition[];
  error: string | null;
};

function App() {
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    void invoke<FeedSnapshot[]>("list_feeds")
      .then(() => {
        setLoadError(null);
      })
      .catch((error) => {
        setLoadError(error instanceof Error ? error.message : String(error));
      })
      .finally(() => {
        setLoading(false);
      });
  }, []);

  return (
    <div className="panel">
      <h1 className="title">Cortado</h1>

      {loading ? <p className="state-msg">Loading native menu…</p> : null}

      {!loading && !loadError ? (
        <p className="state-msg success">
          Tray menu is active. Click the menubar icon to browse feeds and activities.
        </p>
      ) : null}

      {loadError ? (
        <p className="state-msg error">
          Could not load tray data: {loadError}
        </p>
      ) : null}

      <p className="hint">
        Panel UI is disabled in this mode. Use the native macOS menu only.
      </p>
    </div>
  );
}

export default App;
