import { useEffect, useState, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { useFirstLaunch } from "./hooks/useFirstLaunch";
import Welcome from "./pages/WelcomePage";
import Home from "./pages/HomePage";
import Name from "./pages/Name";
import ClipboardGuide from "./pages/ClipboardGuide";
import About from "./pages/About";
import OnlineSearchGuide from "./pages/OnlineSearchGuide";
import OpenFileGuide from "./pages/OpenFileGuide";
import GuideEnd from "./pages/GuideEnd";

function App() {
  const isFirstLaunch = useFirstLaunch();
  // console.log("isFirstLaunch:", isFirstLaunch);

  
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
          setQuery("");
          inputRef.current?.focus();
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

      <BrowserRouter>
      <Routes>
        <Route path="/" element={isFirstLaunch ? <Welcome/> : <Home/>}/>
        <Route path="/home" element={<Home />} />
        <Route path="/welcome" element={<Welcome/>} />
        <Route path="/name" element={<Name/>} />
        <Route path="/About" element={<About/>} />
        <Route path="/ClipboardGuide" element={<ClipboardGuide/>} />
        <Route path="/OnlineSearchGuide" element={<OnlineSearchGuide/>} />
        <Route path="/OpenFileGuide" element={<OpenFileGuide/>} />
        <Route path="/GuideEnd" element={<GuideEnd/>} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
