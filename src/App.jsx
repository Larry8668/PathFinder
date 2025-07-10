import { useEffect, useState, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import HomeOptions from "./components/HomeOptions";
import ClipboardPage from "./components/ClipboardPage";
import OnlineSearchPage from "./components/OnlineSearchPage";
import OpenFilePage from "./components/OpenFilePage";

function App() {
  const [query, setQuery] = useState("");
  const inputRef = useRef(null);
  const [currentPage, setCurrentPage] = useState("home");

  useEffect(() => {
    function handleKeyDown(e) {
      if (e.key === "Escape") {
        if (currentPage === "home") {
          getCurrentWindow().hide();
        } else {
          setCurrentPage("home");
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [currentPage]);

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
          {currentPage === "home" && (
            <HomeOptions
              query={query}
              onSelect={setCurrentPage}
              clearQuery={() => setQuery("")}
            />
          )}
          {currentPage === "clipboard" && <ClipboardPage query={query} />}
          {currentPage === "online-search" && (
            <OnlineSearchPage query={query} />
          )}
          {currentPage === "open-file" && <OpenFilePage query={query} />}
        </div>
      </div>
    </div>
  );
}

export default App;
