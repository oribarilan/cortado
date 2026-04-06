export type StatusKind =
  | "attention-negative"
  | "attention-positive"
  | "waiting"
  | "running"
  | "idle";

export type FieldValue =
  | {
      type: "text";
      value: string;
    }
  | {
      type: "status";
      value: string;
      kind: StatusKind;
    }
  | {
      type: "number";
      value: number;
    }
  | {
      type: "url";
      value: string;
    };

export type Field = {
  name: string;
  label: string;
  value: FieldValue;
};

export type FeedAction = "restart_app";

export type Activity = {
  id: string;
  title: string;
  fields: Field[];
  retained: boolean;
  action?: FeedAction | null;
};

export type FeedSnapshot = {
  name: string;
  feed_type: string;
  activities: Activity[];
  error: string | null;
  hide_when_empty: boolean;
  last_refreshed?: number; // unix ms
  is_disconnected?: boolean;
};
