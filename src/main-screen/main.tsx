import React from "react";
import ReactDOM from "react-dom/client";
import MainScreenApp from "./MainScreenApp";
import "../shared/tokens.css";
import "./main-screen.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MainScreenApp />
  </React.StrictMode>,
);
