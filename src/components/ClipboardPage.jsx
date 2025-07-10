import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";
import Fuse from "fuse.js";

const dummyClips = [
  { text: "Hello world" },
  { text: "Important email copied" },
  { text: "Some other copied text" },
];

const fuse = new Fuse(dummyClips, { keys: ["text"], threshold: 0.4 });

export default function ClipboardPage({ query }) {
  const filtered = query
    ? fuse.search(query).map((res) => res.item)
    : dummyClips;

  const { getItemProps } = useKeyboardNavigation(filtered, (item) => {
    console.log("Selected:", item.text);
  });

  return (
    <div className="option-list">
      {filtered.map((clip, idx) => (
        <div
          {...getItemProps(idx)}
          className={`option-item ${getItemProps(idx).className}`}
          key={idx}
        >
          <span className="icon">📄</span>
          <span>{clip.text}</span>
        </div>
      ))}
    </div>
  );
}
