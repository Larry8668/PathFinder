import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";
import { openUrl } from '@tauri-apps/plugin-opener';
import { invoke } from '@tauri-apps/api/core';
import Fuse from "fuse.js";

const OPTIONS = [
  { title: "Clipboard", icon: "📋", page: "clipboard" },
  { title: "Online Search", icon: "🔍", page: "online-search" },
  { title: "Open App", icon: "📁", page: "open-app" },
  { title: "HLS Screen Share", icon: "📺", page: "hls-screen-share" },
  { title: "Tutorial", icon: "🧭", page: "open-guide" },
];

const fuse = new Fuse(OPTIONS, { keys: ["title"], threshold: 0.4 });
 
export default function HomeOptions({ query, onSelect, clearQuery }) {
  const filtered = query
    ? fuse.search(query).map((result) => result.item)
    : OPTIONS;

  const handleSearch = async (searchQuery) => {
    if (!searchQuery.trim()) return;

    try {
      // Construct search URL (Google search)
      const searchUrl = `https://www.google.com/search?q=${encodeURIComponent(searchQuery)}`;
      
      // Open URL in browser
      await openUrl(searchUrl);
      
      // Hide the PathFinder window
      await invoke('hide_window');
      
      // Clear query
      clearQuery();
    } catch (error) {
      console.error("Failed to open browser:", error);
    }
  };

  // If no matches, show web search option
  const itemsToShow = filtered.length > 0 
    ? filtered 
    : [{ title: `Search online for "${query}"`, icon: "🌐", isWebSearch: true }];

  const { getItemProps } = useKeyboardNavigation(itemsToShow, (item) => {
    if (item.isWebSearch) {
      // Trigger web search
      handleSearch(query);
    } else {
      // Navigate to page
      onSelect(item.page);
      clearQuery();
    }
  });

  return (
    <div className="option-list">
      {itemsToShow.map((opt, idx) => (
        <div
          {...getItemProps(idx)}
          className={`option-item ${getItemProps(idx).className}`}
          key={idx}
        >
          <span className="icon">{opt.icon}</span>
          <span>{opt.title}</span>
        </div>
      ))}
    </div>
  );
}