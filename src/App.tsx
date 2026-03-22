import { useEffect } from "react";

import { invoke } from "@tauri-apps/api/core";

type WatchKind = "status";

type Watch = {
  id: string;
  kind: WatchKind;
  label: string;
  value: string;
  updatedAt: string;
};

type Bean = {
  id: string;
  name: string;
  description: string;
  watches: Watch[];
};

const starterBeans: Bean[] = [
  {
    id: "bean-github-prs",
    name: "GitHub PRs",
    description: "Status watch for open pull requests in personal/cortado.",
    watches: [
      {
        id: "watch-github-pr-status",
        kind: "status",
        label: "review",
        value: "1 awaiting review",
        updatedAt: "just now",
      },
    ],
  },
];

function App() {
  useEffect(() => {
    void invoke("init");
  }, []);

  return (
    <div className="container">
      <h1>Cortado</h1>
      <p className="subtitle">Cross-platform extensible watcher</p>

      <section className="section">
        <h2>Phase 1</h2>
        <ul>
          <li>macOS menubar + panel</li>
          <li>Developer-focused workflows</li>
          <li>Status watches (e.g. GitHub PR status)</li>
        </ul>
      </section>

      <section className="section">
        <h2>Bean model</h2>
        <p>
          A <strong>Bean</strong> is a user-defined item with one or more
          <strong> watch</strong> behaviors.
        </p>
      </section>

      <section className="section">
        <h2>Starter beans</h2>
        <ul className="bean-list">
          {starterBeans.map((bean) => (
            <li className="bean-card" key={bean.id}>
              <p className="bean-name">{bean.name}</p>
              <p className="bean-description">{bean.description}</p>

              {bean.watches.map((watch) => (
                <div className="watch-row" key={watch.id}>
                  <span className="watch-kind">{watch.kind}</span>
                  <span className="watch-label">{watch.label}</span>
                  <span className="watch-value">{watch.value}</span>
                  <span className="watch-meta">updated {watch.updatedAt}</span>
                </div>
              ))}
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

export default App;
