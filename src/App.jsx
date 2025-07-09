import { useEffect, useState, useRef } from "react";
import { register } from "@tauri-apps/plugin-global-shortcut";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

function App() {
  const [query, setQuery] = useState("");
  const boxRef = useRef(null);
  const inputRef = useRef(null);

  useEffect(() => {
    register("Ctrl+Shift+Space", () => {
      console.log("Overlay toggle requested");
    });
  }, []);

  // click outside to close
  useEffect(() => {
    function handleClickOutside(event) {
      if (boxRef.current && !boxRef.current.contains(event.target)) {
        getCurrentWindow().hide();
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div className="raycast-overlay">
      <div className="search-box" ref={boxRef}>
        <input
          autoFocus
          ref={inputRef}
          placeholder="Search..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="search-input"
        />
        <div className="results">
          <p className="empty">Start typing to search your clipboard...</p>
        </div>
      </div>
    </div>
  );
}

export default App;
