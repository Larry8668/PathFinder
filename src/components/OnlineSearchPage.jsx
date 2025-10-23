import { useEffect } from "react";
// 1. Import the correct named function 'openUrl'
import { openUrl } from '@tauri-apps/plugin-opener'; 
import { invoke } from '@tauri-apps/api/core';
import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";

export default function OnlineSearchPage({ query }) {
  const { getItemProps } = useKeyboardNavigation([query], async (item, idx) => {
    await handleSearch(query);
  });

  // 2. Removed the redundant useEffect hook. 
  // 'useKeyboardNavigation' already handles the Enter key.

  const handleSearch = async (searchQuery) => {
    if (!searchQuery.trim()) return;

    try {
      // Construct search URL (Google search)
      const searchUrl = `https://www.google.com/search?q=${encodeURIComponent(searchQuery)}`;
      
      // 3. Call the correct 'openUrl' function
      await openUrl(searchUrl);
      
      // Hide the PathFinder window
      await invoke('hide_window');
    } catch (error) {
      console.error("Failed to open browser:", error);
    }
  };

  return (
    <div className="option-list">
      <div
        {...getItemProps(0)}
        className={`option-item ${getItemProps(0).className}`}
        onClick={() => handleSearch(query)}
      >
        <span className="icon">🌐</span>
        <span>Search online for "{query}"</span>
      </div>
    </div>
  );
}