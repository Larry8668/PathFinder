import { useEffect, useState, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import HomeOptions from "./components/HomeOptions";

function App() {
  const [query, setQuery] = useState("");
  const inputRef = useRef(null);

  useEffect(() => {
    function handleKeyDown(e) {
      if (e.key === "Escape") {
        console.log("Escape key pressed");
        getCurrentWindow().hide();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div className="raycast-overlay">
      <div className="input-wrapper">
        <input
          ref={inputRef}
          placeholder="Search..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="search-input"
        />
      </div>
      <div className="main-container">
        <div className="results">
          <HomeOptions query={query} />
        </div>
      </div>
    </div>
  );
}

export default App;
