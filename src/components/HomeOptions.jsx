import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";

import Fuse from "fuse.js";

const OPTIONS = [
  { title: "Clipboard", icon: "📋", page: "clipboard" },
  { title: "Online Search", icon: "🔍", page: "online-search" },
  { title: "Open File", icon: "📁", page: "open-file" },
];

const fuse = new Fuse(OPTIONS, { keys: ["title"], threshold: 0.4 });

export default function HomeOptions({ query, onSelect, clearQuery }) {
  const filtered = query
    ? fuse.search(query).map((result) => result.item)
    : OPTIONS;

  const { getItemProps } = useKeyboardNavigation(filtered, (item) => {
    onSelect(item.page);
    clearQuery();
  });
  return (
    <div className="option-list">
      {filtered.length ? (
        filtered.map((opt, idx) => (
          <div
            {...getItemProps(idx)}
            className={`option-item ${getItemProps(idx).className}`}
            key={idx}
          >
            <span className="icon">{opt.icon}</span>
            <span>{opt.title}</span>
          </div>
        ))
      ) : (
        <div className="option-item selected">
          <span className="icon">🌐</span>
          <span>Search online for "{query}"</span>
        </div>
      )}
    </div>
  );
}
